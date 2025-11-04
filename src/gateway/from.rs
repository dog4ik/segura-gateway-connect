use crate::{
    connect::{
        self,
        api::{
            RedirectRequest,
            payment::{GwConnectH2HPaymentRequest, H2HCardParams},
        },
    },
    gateway::{self, SeguraOkResponse, status::SeguraStatusResponse},
};

impl<'a> From<&'a connect::api::payment::GwConnectH2HPaymentRequest>
    for super::payin::PaymentInitRequest<'a>
{
    fn from(
        GwConnectH2HPaymentRequest {
            payment,
            params,
            processing_url,
            settings,
            ..
        }: &'a GwConnectH2HPaymentRequest,
    ) -> Self {
        let callback_url = std::env::var("CALLBACK_URL")
            .map(|url| format!("{url}/gateway/callback"))
            .ok();
        super::payin::PaymentInitRequest {
            amount: format!("{:.2}", (payment.gateway_amount as f64 / 100.)),
            currency: &payment.gateway_currency,
            email: params.email.as_deref(),
            country: params.country.as_deref(),
            callback_url,
            return_url: Some(processing_url),
            phone_number: params.phone.as_deref(),
            customer_name: params.first_name.as_deref(),
            customer_id: &settings.client_id,
            // Client reference must be valid uuid
            client_reference: uuid::Uuid::new_v4().to_string(),
            // client_reference: payment.token,
            narration: Some(&payment.product),
            address: params.address.as_deref(),
            payment_method: "card",
            city: params.city.as_deref(),
            state: params.state.as_deref(),
            zip_code: params.postcode.as_deref(),
            ip_address: payment.ip.as_deref(),
        }
    }
}

impl From<(SeguraOkResponse<super::payin::PaymentProcessData>, String)>
    for connect::api::GwConnectH2HPaymentResponse
{
    fn from(
        (value, reference_token): (SeguraOkResponse<super::payin::PaymentProcessData>, String),
    ) -> Self {
        let reference;
        let card_enrolled;
        let redirect_request = match value.data {
            gateway::payin::PaymentProcessData::Standard(standard_payment_data) => {
                tracing::trace!("Segura standard response");
                reference = standard_payment_data.order_reference;
                card_enrolled = false;
                None
            }
            gateway::payin::PaymentProcessData::ThreeDS(three_dspayment_data) => {
                tracing::trace!("Segura 3ds response");
                reference = reference_token;
                card_enrolled = true;
                Some(RedirectRequest {
                    url: three_dspayment_data.redirect.url,
                    kind: connect::api::RedirectRequestType::Get,
                })
            }
        };
        Self {
            redirect_request,
            result: connect::Status::Pending,
            card_enrolled,
            gateway_token: Some(reference),
        }
    }
}

impl From<SeguraOkResponse<super::payin::PaymentInitData>>
    for connect::api::GwConnectH2HPaymentResponse
{
    fn from(value: SeguraOkResponse<super::payin::PaymentInitData>) -> Self {
        let reference = value.data.reference;
        let redirect_request = RedirectRequest {
            url: value
                .data
                .redirect_url
                .expect("redirect url never empty if hosted_payment"),
            kind: connect::api::RedirectRequestType::GetWithProcessing,
        };
        Self {
            redirect_request: Some(redirect_request),
            card_enrolled: false,
            result: connect::Status::Pending,
            gateway_token: Some(reference),
        }
    }
}

impl<'a> super::payin::ProcessRequest<'a> {
    pub fn from(
        GwConnectH2HPaymentRequest {
            params, payment, ..
        }: &'a GwConnectH2HPaymentRequest,
        card_params: &'a H2HCardParams,
        reference: &'a str,
    ) -> Self {
        let expiry = &card_params.expires;
        // Gateway panics when year has 2077 format
        let (expiry_month, year) = expiry
            .split_once('/')
            .expect("reactivepay expires format must be 11/2077");
        let expiry_year = year
            .get(2..)
            .expect("reactivepay expires year must be 4 digits");
        Self {
            pan: &card_params.pan,
            cvv: &card_params.cvv,
            expiry_month,
            expiry_year,
            expiry,
            reference,
            customer_dob: params.birthday.as_deref(),
            cardholder_name: &card_params.holder,
            customer_first_name: params.first_name.as_deref(),
            customer_last_name: params.last_name.as_deref(),
            card_scheme: payment.card_brand_name.as_ref().map(|v| v.to_uppercase()),
            // TODO: pass card type???
            card_type: None,
        }
    }
}

impl From<gateway::SeguraStatus> for connect::Status {
    fn from(value: gateway::SeguraStatus) -> Self {
        match value {
            gateway::SeguraStatus::Failed => Self::Declined,
            gateway::SeguraStatus::Pending => Self::Pending,
            gateway::SeguraStatus::Success => Self::Approved,
        }
    }
}

impl From<SeguraStatusResponse> for connect::status::res::Status {
    fn from(value: SeguraStatusResponse) -> Self {
        Self {
            status: value.data.status.into(),
            details: value.message,
            amount: value.data.amount,
            currency: value.data.currency,
        }
    }
}
