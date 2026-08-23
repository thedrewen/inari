use std::{collections::HashMap, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use getset::Getters;
use mediasoup::{
    consumer::{Consumer, ConsumerOptions}, producer::{Producer, ProducerId, ProducerOptions}, transport::Transport, types::rtp_parameters::{MediaKind, RtpCapabilities, RtpParameters}, webrtc_transport::{WebRtcTransport, WebRtcTransportRemoteParameters}
};
use tokio::sync::{RwLock, mpsc::UnboundedSender};
use uuid::Uuid;
use warp::filters::ws::{Message, WebSocket};

use crate::{
    app::Bundle,
    classes::{
        mediasoup::router::InariRouter,
        ws::{
            requests::{WsRequests, WsRequestsKind},
            responses::{WsResponses, WsResponsesKind},
        },
    },
    utils::logger,
};

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct Client {
    id: usize,
    is_server_client: bool,
    channel: Option<InariRouter>,
    mpsc_tx: Option<UnboundedSender<Message>>,
    code: String,
    bundle: Bundle,
    producer_transport: Option<WebRtcTransport>,
    consumers_transport: Option<WebRtcTransport>,
    is_auth: bool,

    producer: Option<Producer>,
    consumers: Arc<RwLock<HashMap<ProducerId, Consumer>>>
}

impl Client {
    pub fn new(id: usize, bundle: Bundle) -> Self {
        Self {
            id,
            is_server_client: false,
            channel: None,
            mpsc_tx: None,
            code: String::new(),
            bundle,
            consumers_transport: None,
            producer_transport: None,
            is_auth: false,

            producer: None,
            consumers: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn send(&self, response: WsResponses) {
        if let Some(mpsc_tx) = self.mpsc_tx.clone() {
            let _ = mpsc_tx.send(Message::text(serde_json::to_string(&response).unwrap()));
        }
    }

    pub async fn handle(mut self, ws: WebSocket) {
        let (mut tx, mut rx) = ws.split();
        let (mpsc_tx, mut mpsc_rx) = tokio::sync::mpsc::unbounded_channel();
        self.mpsc_tx = Some(mpsc_tx);

        logger::log_info(format!("Client #{} has logged in.", self.id));

        tokio::spawn(async move {
            while let Some(result) = mpsc_rx.recv().await {
                let _ = tx.send(result).await;
            }
        });

        while let Some(result) = rx.next().await {
            match result {
                Ok(msg) => {
                    let value = msg.to_str();
                    if value.is_err() {
                        continue;
                    }
                    let value = value.unwrap();
                    let value = serde_json::from_str(value);
                    if value.is_err() {
                        continue;
                    }
                    let value: WsRequests = value.unwrap();
                    match value.kind {
                        WsRequestsKind::Auth {
                            is_server,
                            auth_key,
                            channel,
                        } => {

                            if self.is_auth {
                                break;
                            }

                            self.is_auth = true;
                            self.is_server_client = is_server;

                            if is_server {
                                if auth_key != self.bundle.config.api_key().clone() {
                                    break;
                                }
                                logger::log_info(format!(
                                    "Client #{} authenticated as server!",
                                    self.id
                                ));
                            } else {
                                logger::log_info(format!(
                                    "Client #{} authenticated as client with code {}!",
                                    self.id, auth_key
                                ));
                                self.code = auth_key;
                            }

                            let router = self.bundle.mediasoup_manager.get_router(channel, &self.bundle.config).await;
                            router.add_client(&self);
                            self.channel = Some(router);

                            self.send(WsResponses {
                                kind: WsResponsesKind::Auth {},
                                transaction_uuid: value.transaction_uuid.clone(),
                            });
                        }

                        // Clients
                        WsRequestsKind::RtpCapabilities {} => {
                            if self.is_connected_as_client()
                                && let Some(channel) = &self.channel
                            {
                                self.send(WsResponses {
                                    kind: WsResponsesKind::RtpCapabilities {
                                        rtp_capabilities: channel.get_rtp_capabilities(),
                                    },
                                    transaction_uuid: value.transaction_uuid.clone(),
                                });
                            }
                        }
                        WsRequestsKind::CreateTransport { is_producer } => {
                            if self.is_connected_as_client()
                                && let Some(channel) = &self.channel
                            {
                                let transport = channel.create_transport(&self).await;
                                let is_success = self.set_transport(transport.clone(), is_producer);
                                if is_success {
                                    self.send(WsResponses {
                                        kind: WsResponsesKind::CreateTransport {
                                            id: transport.id(),
                                            ice_parameters: transport.ice_parameters().clone(),
                                            ice_candidates: transport.ice_candidates().clone(),
                                            dtls_parameters: transport.dtls_parameters().clone(),
                                        },
                                        transaction_uuid: value.transaction_uuid.clone(),
                                    });
                                } else {
                                    break;
                                }
                            }
                        }
                        WsRequestsKind::ConnectTransport {
                            is_producer,
                            dtls_parameters,
                        } => {
                            let transport = if is_producer {
                                &self.producer_transport
                            } else {
                                &self.consumers_transport
                            };

                            if let Some(transport) = transport {
                                let success = transport
                                    .connect(WebRtcTransportRemoteParameters { dtls_parameters })
                                    .await;

                                if success.is_err() {
                                    break;
                                }

                                self.send(WsResponses {
                                    kind: WsResponsesKind::ConnectTransport {},
                                    transaction_uuid: value.transaction_uuid.clone(),
                                });
                            }
                        }
                        WsRequestsKind::Produce {
                            kind,
                            rtp_parameters,
                        } => {
                            if self.is_connected_as_client() {
                                let success = self.produce(kind, rtp_parameters).await;
                                if let Some(producer) = self.producer.clone()
                                    && let Some(channel) = &self.channel
                                    && success
                                {
                                    self.send(WsResponses {
                                        kind: WsResponsesKind::Produce {
                                            id: producer.id(),
                                            details: format!(
                                                "5f671f {}",
                                                env!("CARGO_PKG_VERSION")
                                            ),
                                        },
                                        transaction_uuid: value.transaction_uuid.clone(),
                                    });

                                    let clients_copy = channel.clients().read().await;
                                    for client in clients_copy.values() {
                                        if *client.id() != self.id {
                                            let uuid = Uuid::new_v4().to_string();
                                            client.send(WsResponses {
                                                kind: WsResponsesKind::AddClients {
                                                    ids: vec![producer.id()],
                                                },
                                                transaction_uuid: uuid,
                                            });
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        WsRequestsKind::Consume {
                            producer_id,
                            rtp_capabilities,
                        } => {
                            if self.is_connected_as_client() {
                                let consumer = self.consume(producer_id, rtp_capabilities).await;
                                if let Some(consumer) = consumer {
                                    self.send(WsResponses { kind: WsResponsesKind::Consume { consumer_id: consumer.id(), kind: consumer.kind(), rtp_parameters: consumer.rtp_parameters().clone() }, transaction_uuid: value.transaction_uuid.clone() });
                                }
                            }
                        }
                        WsRequestsKind::FetchClients {} => {
                            if self.is_connected_as_client() && let Some(channel) = &self.channel {
                                let clients = channel.clients().read().await.clone();
                                let mut ids: Vec<ProducerId> = vec![];
                                for client in clients.values() {
                                    if let Some(producer) = client.producer() {
                                        ids.push(producer.id());
                                    }
                                }

                                self.send(
                                    WsResponses { kind: WsResponsesKind::AddClients { ids }, transaction_uuid: value.transaction_uuid.clone() }
                                );
                            }
                        }

                        // Servers
                        WsRequestsKind::Clients { clients } => {
                            if self.is_server_client && let Some(channel) = &self.channel {
                                channel.handle_client_as_server(clients).await;
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        if let Some(channel) = &self.channel {
            channel.remove_client(&self).await;
        }

        logger::log_info(format!("Client #{} has logged out.", self.id));
    }

    pub fn is_connected_as_client(&self) -> bool {
        !self.is_server_client && self.channel.is_some()
    }

    pub fn set_transport(&mut self, transport: WebRtcTransport, is_producer: bool) -> bool {
        if is_producer && self.producer_transport.is_none() {
            self.producer_transport = Some(transport);
            return true;
        } else if !is_producer && self.consumers_transport.is_none() {
            self.consumers_transport = Some(transport);
            return true;
        }

        false
    }

    pub async fn produce(&mut self, kind: MediaKind, rtp_parameters: RtpParameters) -> bool {
        if let Some(transport) = self.producer_transport.clone() && self.producer.is_none() {
            let result = transport
                .produce(ProducerOptions::new(kind, rtp_parameters))
                .await;

            if let Some(producer) = result.ok()
                && let Some(channel) = &self.channel
            {
                let _ = producer.resume().await;
                self.producer = Some(producer);
                channel.register_client(self.clone()).await;

                return true;
            }
        }

        false
    }

    pub async fn consume(&mut self, producer_id: ProducerId, rtp_capabilities: RtpCapabilities) -> Option<Consumer> {
        if let Some(transport) = &self.consumers_transport {
            let consumer = transport.consume(ConsumerOptions::new(producer_id, rtp_capabilities)).await;

            if let Some(consumer) = consumer.ok() {
                self.consumers.write().await.insert(producer_id, consumer.clone());
                let _ = consumer.pause().await;
                return Some(consumer);
            }   
        }

        None
    }


    pub async fn mute_consumers(&self) {
        for consumer in self.consumers.read().await.values() {
            let _ = consumer.pause().await;
        }
    }

    pub async fn mute_consumer(&self, producer_id: ProducerId) {
        if let Some(consumer) = self.consumers.read().await.get(&producer_id) {
            let _ = consumer.pause().await;
        } 
    }

    pub async fn resume_consumer(&self, producer_id: ProducerId) {
        if let Some(consumer) = self.consumers.read().await.get(&producer_id) {
            let _ = consumer.resume().await;
        } 
    }
}
