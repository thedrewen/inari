use mediasoup::{consumer::ConsumerId, producer::ProducerId, transport::TransportId, types::{data_structures::{DtlsParameters, IceCandidate, IceParameters}, rtp_parameters::{MediaKind, RtpCapabilitiesFinalized, RtpParameters}}};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ClientLocalState {
    pub position: [f64; 3],
    pub display_name: String,
    pub producer_id: ProducerId
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ClientState {
    pub display_name: String,
    pub producer_id: ProducerId
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ConnectionDefinition {
    pub code: String,
    pub display_name: String,
    pub is_connector: bool,
    pub is_force_muted: bool,
    pub is_force_deafened: bool,
    pub is_muted: bool,
    pub is_deafened: bool
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
    // All
    Auth { max_distance: f64, app_version: String },
    Error { message: String, details: String, code: u32 },
    
    // Clients
    RtpCapabilities { rtp_capabilities: RtpCapabilitiesFinalized },
    CreateTransport { id: TransportId, ice_parameters: IceParameters, ice_candidates: Vec<IceCandidate>, dtls_parameters: DtlsParameters },
    ConnectTransport {},
    Produce { id: ProducerId, details: String },
    AddClients { clients: Vec<ClientState> },
    RemoveClient { id: ProducerId },
    Consume { consumer_id: ConsumerId, kind: MediaKind, rtp_parameters: RtpParameters },
    SyncClientState { is_force_muted: bool, is_force_deafened: bool, is_muted: bool, is_deafened: bool, orientation: [f64; 3], local_clients: Vec<ClientLocalState>, display_name: String },
    
    // Connectors
    SyncConnectorState { connections: Vec<ConnectionDefinition> }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsResponses {
    #[serde(flatten)]
    pub kind: WsResponsesKind,
    pub transaction_uuid: String,
}