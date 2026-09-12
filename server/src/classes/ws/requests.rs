use mediasoup::{producer::ProducerId, types::{data_structures::DtlsParameters, rtp_parameters::{MediaKind, RtpCapabilities, RtpParameters}}};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientServerDefinition {
    pub code: String,
    pub position: [f64; 3],
    pub muted: bool
}

impl ClientServerDefinition {
    pub fn distance_to(&self, target: &ClientServerDefinition) -> f64 {
        let dx = self.position[0] - target.position[0];
        let dy = self.position[1] - target.position[1];
        let dz = self.position[2] - target.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn related_position(&self, target: &ClientServerDefinition) -> [f64; 3] {
        [target.position[0] - self.position[0], target.position[1] - self.position[1], target.position[2] - self.position[2]]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde( tag = "kind", content = "data", rename_all = "snake_case" )]
pub enum WsRequestsKind {
    Auth { is_server: bool, auth_key: String, channel: String },
    
    // Clients
    RtpCapabilities {  },
    CreateTransport { is_producer: bool },
    ConnectTransport { is_producer: bool, dtls_parameters: DtlsParameters },
    Produce { kind: MediaKind, rtp_parameters: RtpParameters },
    FetchClients { },
    Consume { producer_id: ProducerId, rtp_capabilities: RtpCapabilities },

    // Server
    Clients { clients: Vec<ClientServerDefinition> }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsRequests {
    #[serde(flatten)]
    pub kind: WsRequestsKind,
    pub transaction_uuid: String,
}   