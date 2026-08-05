use mediasoup::types::{data_structures::DtlsParameters, rtp_parameters::{MediaKind, RtpParameters}};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde( tag = "kind", content = "data", rename_all = "snake_case" )]
pub enum WsRequestsKind {
    Link { code: String },
    CreateProducerTransport { },
    ConnectProducerTransport { dtls_parameters: DtlsParameters },
    Produce { kind: MediaKind, rtp_parameters: RtpParameters }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsRequests {
    #[serde(flatten)]
    pub kind: WsRequestsKind,
    pub transaction_uuid: String,
}