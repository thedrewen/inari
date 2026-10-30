use std::{collections::HashMap, sync::Arc, time::Duration, vec};

use futures_util::{SinkExt, StreamExt};
use getset::Getters;
use mediasoup::{
    consumer::{Consumer, ConsumerOptions}, producer::{Producer, ProducerId, ProducerOptions}, transport::Transport, types::rtp_parameters::{MediaKind, RtpCapabilities, RtpParameters}, webrtc_transport::{WebRtcTransport, WebRtcTransportRemoteParameters}
};
use tokio::{sync::{RwLock, mpsc::UnboundedSender}, time::timeout};
use uuid::Uuid;
use warp::filters::ws::{Message, WebSocket};

use crate::{
    app::Bundle, classes::{
        mediasoup::router::InariRouter, ws::{
            requests::{WsRequests, WsRequestsKind}, responses::{Connection, ErrorType, WsResponses, WsResponsesKind},
        },
    }, utils::logger,
};

const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

struct LocalConsumer {
    consumer: Consumer,
    muted: bool
}

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct Client {
    id: usize,
    is_connector: bool,
    channel: Option<InariRouter>,
    mpsc_tx: Option<UnboundedSender<Message>>,
    code: String,
    bundle: Bundle,
    producer_transport: Option<WebRtcTransport>,
    consumers_transport: Option<WebRtcTransport>,
    is_auth: bool,

    producer: Option<Producer>,
    consumers: Arc<RwLock<HashMap<ProducerId, LocalConsumer>>>,

    // Connector Data
    head_orientation: Arc<RwLock<[f64; 3]>>,
    force_muted: Arc<RwLock<bool>>,
    muted: Arc<RwLock<bool>>,
    daefen: Arc<RwLock<bool>>
}

impl Client {
    pub fn new(id: usize, bundle: Bundle) -> Self {
        Self {
            id,
            is_connector: false,
            channel: None,
            mpsc_tx: None,
            code: String::new(),
            bundle,
            consumers_transport: None,
            producer_transport: None,
            is_auth: false,

            producer: None,
            consumers: Arc::new(RwLock::new(HashMap::new())),

            head_orientation: Arc::new(RwLock::new([0.0, 0.0, 0.0])),
            force_muted: Arc::new(RwLock::new(true)),
            muted: Arc::new(RwLock::new(false)),
            daefen: Arc::new(RwLock::new(false))
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

        while let Ok(Some(result)) = timeout(IDLE_TIMEOUT, rx.next()).await {
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
                            is_connector,
                            auth_key,
                            channel,
                        } => {

                            if self.is_auth {
                                self.send(WsResponses { kind: WsResponsesKind::Error { message: ErrorType::AlreadyConnected.to_str(), details: "You are already logged in.".to_string(), code: 409 }, transaction_uuid: value.transaction_uuid.clone() });
                                break;
                            }

                            self.is_auth = true;
                            self.is_connector = is_connector;

                            let router = self.bundle.mediasoup_manager.get_router(channel, &self.bundle.config).await;
                            if is_connector {
                                if auth_key != self.bundle.config.api_key().clone() {
                                    router.remove_client(&self).await;
                                    self.send(WsResponses { kind: WsResponsesKind::Error { message: ErrorType::Unauthorized.to_str(), details: "The API key provided is invalid.".to_string(), code: 401 }, transaction_uuid: value.transaction_uuid.clone() });
                                    break;
                                }
                                logger::log_info(format!(
                                    "Client #{} authenticated as server!",
                                    self.id
                                ));
                            } else {
                                if router.has_client_with_code(auth_key.clone()).await || auth_key.len() < 6 {
                                    router.remove_client(&self).await;
                                    self.send(WsResponses { kind: WsResponsesKind::Error { message: ErrorType::BadRequest.to_str(), details: "The code provided is invalid.".to_string(), code: 400 }, transaction_uuid: value.transaction_uuid.clone() });
                                    break;
                                }
                                logger::log_info(format!(
                                    "Client #{} authenticated as client with code {}!",
                                    self.id, auth_key
                                ));
                                self.code = auth_key;
                            }

                            router.add_client(&self).await;
                            let max_distance = *router.config().max_distance();
                            let app_version = router.config().app_version().clone();
                            self.channel = Some(router);

                            self.send(WsResponses {
                                kind: WsResponsesKind::Auth { max_distance, app_version },
                                transaction_uuid: value.transaction_uuid.clone(),
                            });
                        }
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
                        WsRequestsKind::SyncClientState { is_mute, is_daefen } => {
                            *self.muted.write().await = is_mute;
                            *self.daefen.write().await = is_daefen;
                        }

                        // Connectors
                        WsRequestsKind::SyncConnectorState { clients } => {
                            if self.is_connector && let Some(channel) = &self.channel {
                                channel.handle_client_as_server(clients).await;

                                let mut connections: Vec<Connection> = vec![];
                                let clients = channel.clients().read().await.clone();
                                for client in clients.values() {
                                    let connection = Connection { code: client.code().clone(), force_muted: client.force_muted.read().await.clone(), is_connector: *client.is_connector(), is_mute: client.muted.read().await.clone(), is_daefen: client.daefen.read().await.clone() };
                                    connections.push(connection);
                                }
                                if connections.len() > 0 {
                                    self.send(WsResponses { kind: WsResponsesKind::SyncConnectorState { connections }, transaction_uuid: value.transaction_uuid.clone() });
                                }
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
        !self.is_connector && self.channel.is_some()
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
                self.consumers.write().await.insert(producer_id, LocalConsumer { consumer: consumer.clone(), muted: true });
                let _ = consumer.pause().await;
                return Some(consumer);
            }   
        }

        None
    }


    pub async fn mute_consumers(&self) {
        for consumer in self.consumers.write().await.values_mut() {
            if !consumer.muted {
                let _ = consumer.consumer.pause().await;
                consumer.muted = true;
            }
        }
    }

    pub async fn mute_consumer(&self, producer_id: ProducerId) {
        if let Some(consumer) = self.consumers.write().await.get_mut(&producer_id) {
            if !consumer.muted {
                let _ = consumer.consumer.pause().await;
                consumer.muted = true;
            }
        } 
    }

    pub async fn resume_consumer(&self, producer_id: ProducerId) {
        if let Some(consumer) = self.consumers.write().await.get_mut(&producer_id) {
            if consumer.muted {
                let _ = consumer.consumer.resume().await;
                consumer.muted = false;
            }
        } 
    }
}
