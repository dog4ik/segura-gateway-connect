use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum_extra::headers::{self, HeaderMapExt};
use base64::{Engine, prelude::BASE64_STANDARD};

pub fn authenticated_headers(client_id: &str, secret: &str) -> HeaderMap {
    let auth = BASE64_STANDARD.encode(format!("{client_id}:{secret}"));
    let mut map = HeaderMap::new();
    map.insert(
        HeaderName::from_static("authkey"),
        HeaderValue::from_str(&auth).expect("header value is ascii"),
    );
    map.typed_insert(headers::ContentType::json());
    map
}
