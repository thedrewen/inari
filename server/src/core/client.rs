use std::{collections::HashMap};

use mediasoup::{
    consumer::{Consumer, ConsumerOptions}, producer::{Producer, ProducerId, ProducerOptions}, transport::Transport, types::{
        data_structures::DtlsParameters, rtp_parameters::{MediaKind, RtpCapabilities, RtpParameters},
    }, webrtc_transport::{WebRtcTransport, WebRtcTransportRemoteParameters},
};
use tokio::sync::mpsc::UnboundedSender;
use warp::filters::ws::Message;

use crate::core::responses::{
    WsResponses, WsResponsesKind::{ConnectTransport, Consume, CreateTransport, Produce},
};

#[derive(Clone)]
pub struct Client {
    id: usize,
    mpsc_tx: UnboundedSender<Message>,
    connected: bool,
    connected_as_server: bool,
    pub code: String,

    producer: Option<Producer>,
    producer_id: Option<ProducerId>,
    producer_transport: Option<WebRtcTransport>,
    consumers_transport: Option<WebRtcTransport>,
    consumers: HashMap<ProducerId, Consumer>
}

impl Client {
    pub fn new(id: usize, mpsc_tx: UnboundedSender<Message>) -> Self {
        Self {
            id,
            mpsc_tx,
            connected: false,
            connected_as_server: false,
            code: String::new(),

            producer: None,
            producer_id: None,
            producer_transport: None,
            consumers_transport: None,
            consumers: HashMap::new()
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn set_connected(&mut self) {
        self.connected = true;
    }

    pub fn is_connected_as_server(&self) -> bool {
        self.connected_as_server
    }

    pub fn set_connected_as_server(&mut self) {
        self.connected_as_server = true;
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_producer_id(&self) -> Option<ProducerId> {
        self.producer_id.clone()
    }

    pub fn send(&self, response: WsResponses) {
        let _ = self
            .mpsc_tx
            .send(Message::text(serde_json::to_string(&response).unwrap()));
    }

    pub fn set_transport(&mut self, is_producer: bool, transport: WebRtcTransport, transaction_uuid: String) {
        
        if self.producer_transport.is_none() && is_producer {
            self.producer_transport = Some(transport.clone());

            self.send(WsResponses {
                kind: CreateTransport {
                    id: transport.id(),
                    ice_parameters: transport.ice_parameters().clone(),
                    ice_candidates: transport.ice_candidates().clone(),
                    dtls_parameters: transport.dtls_parameters(),
                },
                transaction_uuid,
            });
        } else if self.consumers_transport.is_none() {
            self.consumers_transport = Some(transport.clone());
            
            self.send(WsResponses {
                kind: CreateTransport {
                    id: transport.id(),
                    ice_parameters: transport.ice_parameters().clone(),
                    ice_candidates: transport.ice_candidates().clone(),
                    dtls_parameters: transport.dtls_parameters(),
                },
                transaction_uuid,
            });
        }
    }

    pub async fn connect_transport(
        &mut self, 
        is_producer: bool,
        dtls_parameters: DtlsParameters,
        transaction_uuid: String,
    ) {
        let transport = if is_producer {
            &self.producer_transport
        } else {
            &self.consumers_transport
        }; 
        
        if let Some(transport) = transport {
            let _ = transport
                .connect(WebRtcTransportRemoteParameters { dtls_parameters })
                .await;

            self.send(WsResponses { kind: ConnectTransport {  }, transaction_uuid });
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
            let _ = producer.resume().await;

            self.send(WsResponses {
                kind: Produce { id: producer.id(), details: format!("5f671f {}", env!("CARGO_PKG_VERSION")) },
                transaction_uuid,
            });

            self.producer_id = Some(producer.id());
            self.producer = Some(producer);
            return self.producer.clone();
        }
        None
    }

    pub async fn consume(&mut self, producer_id: ProducerId, rtp_capabilities: RtpCapabilities, transaction_uuid: String) -> Option<Consumer>  {
        if let Some(transport) = &self.consumers_transport {
            let consumer = transport.consume(ConsumerOptions::new(producer_id, rtp_capabilities)).await.unwrap();
            self.consumers.insert(producer_id, consumer.clone());
            let _ = consumer.resume().await;
            self.send(WsResponses { kind: Consume { consumer_id: consumer.id().clone(), kind: consumer.kind().clone(), rtp_parameters: consumer.rtp_parameters().clone() }, transaction_uuid });
            return Some(consumer);
        }
        None
    }

    pub fn remove_consumer(&mut self, producer_id: ProducerId) {
        self.consumers.remove(&producer_id);
    }
}
