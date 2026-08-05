use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde( tag = "kind", content = "data", rename_all = "snake_case" )]
pub enum WsRequestsKind {
    Link { code: String },
    CreateProducerTransport { }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsRequests {
    #[serde(flatten)]
    pub kind: WsRequestsKind,
    pub transaction_uuid: String,
}