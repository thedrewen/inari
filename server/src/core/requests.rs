use mediasoup::{producer::ProducerId, types::{data_structures::DtlsParameters, rtp_parameters::{MediaKind, RtpCapabilities, RtpParameters}}};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde( tag = "kind", content = "data", rename_all = "snake_case" )]
pub enum WsRequestsKind {
    Link { code: String },
    CreateTransport { is_producer: bool },
    ConnectTransport { is_producer: bool, dtls_parameters: DtlsParameters },
    Produce { kind: MediaKind, rtp_parameters: RtpParameters },
    FetchClients { },
    Consume { producer_id: ProducerId, rtp_capabilities: RtpCapabilities }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsRequests {
    #[serde(flatten)]
    pub kind: WsRequestsKind,
    pub transaction_uuid: String,
}   