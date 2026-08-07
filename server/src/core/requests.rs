use mediasoup::{producer::ProducerId, types::{data_structures::DtlsParameters, rtp_parameters::{MediaKind, RtpCapabilities, RtpParameters}}};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientServerDefinition {
    pub code: String,
    pub position: [usize; 3]
}

impl ClientServerDefinition {
    pub fn distance_to(&self, target: &ClientServerDefinition) -> f64 {
        let dx = self.position[0] as f64 - target.position[0] as f64;
        let dy = self.position[1] as f64 - target.position[1] as f64;
        let dz = self.position[2] as f64 - target.position[2] as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde( tag = "kind", content = "data", rename_all = "snake_case" )]
pub enum WsRequestsKind {
    // Clients
    RtpCapabilities {  },
    Link { code: String },
    CreateTransport { is_producer: bool },
    ConnectTransport { is_producer: bool, dtls_parameters: DtlsParameters },
    Produce { kind: MediaKind, rtp_parameters: RtpParameters },
    FetchClients { },
    Consume { producer_id: ProducerId, rtp_capabilities: RtpCapabilities },

    // Server
    Auth { api_key: String },
    Clients { clients: Vec<ClientServerDefinition> }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsRequests {
    #[serde(flatten)]
    pub kind: WsRequestsKind,
    pub transaction_uuid: String,
}   