use mediasoup::types::rtp_parameters::RtpCapabilitiesFinalized;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde( tag = "kind", content = "data", rename_all = "snake_case" )]
pub enum WsResponsesKind {
    RtpCapabilities { rtp_capabilities: RtpCapabilitiesFinalized } 
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsResponses {
    #[serde(flatten)]
    pub kind: WsResponsesKind,
    pub transaction_uuid: String,
}