pub mod req {
    use serde::Deserialize;

    use crate::connect::api::payment::Settings;

    #[derive(Debug, Deserialize)]
    pub struct Request {
        pub payment: Payment,
        pub settings: Settings,
    }

    #[derive(Debug, Deserialize)]
    pub struct Payment {
        pub gateway_token: String,
        pub token: String,
    }
}

pub mod res {
    use serde::Serialize;

    use crate::connect;

    #[derive(Debug, Serialize)]
    pub struct Status {
        /// Current status of the transaction (e.g. pending, approved)
        pub status: connect::Status,
        /// Additional status details or message
        pub details: String,
        /// Amount of the transaction in minor units
        pub amount: usize,
        pub currency: String,
    }
}
