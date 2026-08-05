use mediasoup::{transport::TransportId, types::{data_structures::{DtlsParameters, IceCandidate, IceParameters}, rtp_parameters::RtpCapabilitiesFinalized}};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde( tag = "kind", content = "data", rename_all = "snake_case" )]
pub enum WsResponsesKind {
    RtpCapabilities { rtp_capabilities: RtpCapabilitiesFinalized },
    CreateProducerTransport { id: TransportId, ice_parameters: IceParameters, ice_candidates: Vec<IceCandidate>, dtls_parameters: DtlsParameters }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsResponses {
    #[serde(flatten)]
    pub kind: WsResponsesKind,
    pub transaction_uuid: String,
}