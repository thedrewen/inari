use mediasoup::{consumer::ConsumerId, producer::ProducerId, transport::TransportId, types::{data_structures::{DtlsParameters, IceCandidate, IceParameters}, rtp_parameters::{MediaKind, RtpCapabilitiesFinalized, RtpParameters}}};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde( tag = "kind", content = "data", rename_all = "snake_case" )]
pub enum WsResponsesKind {
    Welcome { app_version: String },
    
    // Client
    RtpCapabilities { rtp_capabilities: RtpCapabilitiesFinalized },
    CreateTransport { id: TransportId, ice_parameters: IceParameters, ice_candidates: Vec<IceCandidate>, dtls_parameters: DtlsParameters },
    ConnectTransport {},
    Produce { id: ProducerId, details: String },
    AddClients { ids: Vec<ProducerId> },
    RemoveClient { id: ProducerId },
    Consume { consumer_id: ConsumerId, kind: MediaKind, rtp_parameters: RtpParameters },
    ConsumerRelatedPosition { position: [f64; 3], producer_id: ProducerId },
    ProducerHead { orientation: [f64; 3] },
    
    // Server
    Auth { }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsResponses {
    #[serde(flatten)]
    pub kind: WsResponsesKind,
    pub transaction_uuid: String,
}