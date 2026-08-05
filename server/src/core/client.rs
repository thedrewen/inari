use mediasoup::{
    producer::{Producer, ProducerOptions},
    transport::Transport,
    types::{
        data_structures::DtlsParameters,
        rtp_parameters::{MediaKind, RtpParameters},
    },
    webrtc_transport::{WebRtcTransport, WebRtcTransportRemoteParameters},
};
use tokio::sync::mpsc::UnboundedSender;
use warp::filters::ws::Message;

use crate::core::responses::{
    WsResponses, WsResponsesKind::{ConnectProducerTransport, CreateProducerTransport, Produce},
};

#[derive(Clone)]
pub struct Client {
    id: usize,
    mpsc_tx: UnboundedSender<Message>,
    connected: bool,
    pub code: String,

    producer: Option<Producer>,
    producer_transport: Option<WebRtcTransport>,
}

impl Client {
    pub fn new(id: usize, mpsc_tx: UnboundedSender<Message>) -> Self {
        Self {
            id,
            mpsc_tx,
            connected: false,
            code: String::new(),

            producer: None,
            producer_transport: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn set_connected(&mut self) {
        self.connected = true;
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn send(&self, response: WsResponses) {
        let _ = self
            .mpsc_tx
            .send(Message::text(serde_json::to_string(&response).unwrap()));
    }

    pub fn set_producer_transport(&mut self, transport: WebRtcTransport, transaction_uuid: String) {
        self.producer_transport = Some(transport.clone());

        self.send(WsResponses {
            kind: CreateProducerTransport {
                id: transport.id(),
                ice_parameters: transport.ice_parameters().clone(),
                ice_candidates: transport.ice_candidates().clone(),
                dtls_parameters: transport.dtls_parameters(),
            },
            transaction_uuid,
        });
    }

    pub async fn connect_producer_transport(
        &mut self,
        dtls_parameters: DtlsParameters,
        transaction_uuid: String,
    ) {
        if let Some(transport) = &self.producer_transport {
            let _ = transport
                .connect(WebRtcTransportRemoteParameters { dtls_parameters })
                .await;

            self.send(WsResponses { kind: ConnectProducerTransport {  }, transaction_uuid });
        }
    }

    pub async fn produce(
        &mut self,
        kind: MediaKind,
        rtp_parameters: RtpParameters,
        transaction_uuid: String,
    ) -> Option<Producer> {
        if let Some(transport) = &self.producer_transport {
            let result = transport
                .produce(ProducerOptions::new(kind, rtp_parameters))
                .await;

            if result.is_err() {
                return None;
            }
            let producer = result.unwrap();

            self.send(WsResponses {
                kind: Produce { id: producer.id() },
                transaction_uuid,
            });

            self.producer = Some(producer);
            return self.producer.clone();
        }
        None
    }
}
