use aes::cipher::block_padding;
use base64::{Engine, engine::general_purpose};
use serde::{Deserialize, Serialize};

use crate::connect::callback::CallbackPayload;

#[derive(Serialize, Deserialize)]
struct SecureBlock {
    encrypted_data: String,
    iv_value: String,
}

#[derive(Serialize)]
struct JwtPayload<'a> {
    #[serde(flatten)]
    payload: &'a CallbackPayload,
    secure: SecureBlock,
}

fn gen_iv() -> [u8; 16] {
    use rand::TryRngCore;
    let mut iv = [0u8; 16];
    rand::rngs::OsRng.try_fill_bytes(&mut iv).unwrap();
    iv
}

fn encrypt_merchant_key(
    merchant_key: &str,
    sign_key: [u8; 32],
    iv: [u8; 16],
) -> anyhow::Result<(String, String)> {
    use aes::cipher::{BlockEncryptMut, KeyIvInit};

    let res = cbc::Encryptor::<aes::Aes256>::new(&sign_key.into(), &iv.into())
        .encrypt_padded_vec_mut::<block_padding::Pkcs7>(merchant_key.as_bytes());
    let encrypted_data = general_purpose::STANDARD.encode(&res);
    let iv_base64 = general_purpose::STANDARD.encode(iv);
    Ok((encrypted_data, iv_base64))
}

// Function to create the JWT
pub fn create_jwt(
    payload: &CallbackPayload,
    merchant_key: &str,
    sign_key: &[u8; 32],
) -> anyhow::Result<String> {
    let iv = gen_iv();
    // Encrypt the merchant key
    let (encrypted_data, iv_value) = encrypt_merchant_key(merchant_key, *sign_key, iv)?;

    // Create the JWT payload
    let payload = JwtPayload {
        payload,
        secure: SecureBlock {
            encrypted_data,
            iv_value,
        },
    };

    let token = j::encode(&payload, sign_key)?;
    Ok(token)
}

/// I am not sure this implementation of jwt is correct. Consider using proper crate.
mod j {
    use base64::prelude::*;
    use hmac::{Hmac, Mac};
    use serde::Serialize;
    use sha2::Sha512;

    type HmacSha512 = Hmac<Sha512>;

    #[derive(Debug, Serialize)]
    struct Header {
        alg: &'static str,
        typ: &'static str,
    }

    impl Header {
        fn sha512() -> Self {
            Self {
                alg: "HS512",
                typ: "JWT",
            }
        }
    }

    pub fn encode(payload: impl Serialize, sign_key: &[u8; 32]) -> anyhow::Result<String> {
        let header = Header::sha512();
        let payload = payload;
        let mut result = String::new();
        result.push_str(&BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_string(&header)?));
        result.push('.');
        result.push_str(&BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_string(&payload)?));
        let mut mac: HmacSha512 = HmacSha512::new_from_slice(sign_key).unwrap();
        mac.update(result.as_bytes());
        let sig: &[u8] = &mac.finalize().into_bytes();
        result.push('.');
        result.push_str(&BASE64_URL_SAFE_NO_PAD.encode(sig));
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose};

    use crate::connect::callback::{CallbackStatus, jwt::encrypt_merchant_key};

    #[test]
    fn encode_jwt() {
        let merchant_key = "zlfhcrecevingrxrlsbezrepunag";
        let payload = super::CallbackPayload {
            status: CallbackStatus::Approved,
            currency: "RUB".into(),
            amount: 100,
        };
        super::create_jwt(&payload, merchant_key, b"vhfrnepuogjvhfrarbivzogjvhfrehfg").unwrap();
    }

    #[test]
    fn encrypt_private_key() {
        let merchant_key = "5178831496700b3634e4";
        let sign_key: [u8; 32] = TryFrom::try_from(*b"e7403b3c0d76a35312e7cc65eeb75808").unwrap();
        let iv: [u8; 16] =
            TryFrom::try_from(hex::decode("293c20e6038619aa40d774f4fc6934f2").unwrap()).unwrap();
        let expected = "cZu0SLPSItrNtfG8hVIz24Dc6eHW1Ujj19LFGD7t6yk=";
        let (output_encrypted, output_iv) =
            encrypt_merchant_key(merchant_key, sign_key, iv).unwrap();
        assert_eq!(output_encrypted, expected);
        assert_eq!(output_iv, general_purpose::STANDARD.encode(&iv));
    }
}
