use mediasoup::{consumer::ConsumerId, producer::ProducerId, transport::TransportId, types::{data_structures::{DtlsParameters, IceCandidate, IceParameters}, rtp_parameters::{MediaKind, RtpCapabilitiesFinalized, RtpParameters}}};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ConsumerPosition {
    pub position: [f64; 3],
    pub producer_id: ProducerId
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Connection {
    pub code: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ErrorType {
    AlreadyConnected,
    Unauthorized,
    BadRequest
}

impl ErrorType {
    pub fn to_str(&self) -> String {
        match self {
            ErrorType::AlreadyConnected => "ALREADY_CONNECTED".to_string(),
            ErrorType::Unauthorized => "UNAUTHORIZED".to_string(),
            ErrorType::BadRequest => "BAD_REQUEST".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde( tag = "kind", content = "data", rename_all = "snake_case" )]
pub enum WsResponsesKind {
    RtpCapabilities { rtp_capabilities: RtpCapabilitiesFinalized },
    CreateTransport { id: TransportId, ice_parameters: IceParameters, ice_candidates: Vec<IceCandidate>, dtls_parameters: DtlsParameters },
    ConnectTransport {},
    Produce { id: ProducerId, details: String },
    AddClients { ids: Vec<ProducerId> },
    RemoveClient { id: ProducerId },
    Consume { consumer_id: ConsumerId, kind: MediaKind, rtp_parameters: RtpParameters },
    SyncClientState { force_muted: bool, orientation: [f64; 3], consumers: Vec<ConsumerPosition> },
    Auth { max_distance: f64, app_version: String },
    Error { message: String, details: String, code: u32 },
    
    // Server
    SyncServerState { connections: Vec<Connection> }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsResponses {
    #[serde(flatten)]
    pub kind: WsResponsesKind,
    pub transaction_uuid: String,
}