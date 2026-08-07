use std::{
    collections::HashMap,
    env,
    net::Ipv4Addr,
    num::{NonZeroU8, NonZeroU32},
    sync::Arc,
};

use mediasoup::{
    consumer::Consumer,
    producer::{Producer, ProducerId},
    router::{Router, RouterOptions},
    types::{
        data_structures::{DtlsParameters, ListenInfo},
        rtp_parameters::{
            MediaKind, MimeTypeAudio, RtpCapabilities, RtpCapabilitiesFinalized,
            RtpCodecCapability, RtpCodecParametersParameters, RtpParameters,
        },
    },
    webrtc_server::{WebRtcServer, WebRtcServerListenInfos, WebRtcServerOptions},
    webrtc_transport::{WebRtcTransport, WebRtcTransportOptions},
    worker::WorkerSettings,
    worker_manager::WorkerManager,
};
use rand::{RngExt, distr::Alphanumeric};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    core::{
        client::Client,
        requests::ClientServerDefinition,
        responses::{
            WsResponses,
            WsResponsesKind::{AddClients, RemoveClient},
        },
    },
    logger,
};

#[derive(Clone)]
pub struct Manager {
    clients: Arc<RwLock<HashMap<usize, Client>>>,
    producers: Arc<RwLock<HashMap<ProducerId, usize>>>,
    router: Router,
    webrtc_server: WebRtcServer,
    api_key: String,
}

impl Manager {
    pub async fn new() -> Self {
        let w_manager = WorkerManager::new();
        let worker = w_manager
            .create_worker(WorkerSettings::default())
            .await
            .unwrap();

        let port: u16 = env::var("WEBRTC_SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(44444);

        let listen_info = ListenInfo {
            protocol: mediasoup::types::data_structures::Protocol::Udp,
            ip: std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            announced_address: Some(
                env::var("SERVER_PUBLIC_IP").unwrap_or(String::from("127.0.0.1")),
            ),
            port: Some(port),
            port_range: None,
            expose_internal_ip: false,
            flags: None,
            recv_buffer_size: None,
            send_buffer_size: None,
        };

        let listen_infos = WebRtcServerListenInfos::new(listen_info);
        let webrtc_server_option = WebRtcServerOptions::new(listen_infos);
        let webrtc_server = worker
            .create_webrtc_server(webrtc_server_option)
            .await
            .unwrap();

        logger::log_info(format!("Webrtc Server running on : {}", port));

        let audio_codec = RtpCodecCapability::Audio {
            mime_type: MimeTypeAudio::Opus,
            preferred_payload_type: None,
            clock_rate: NonZeroU32::new(48000).unwrap(),
            channels: NonZeroU8::new(2).unwrap(),
            parameters: RtpCodecParametersParameters::from([
                ("useinbandfec", 1u32.into()),
                ("minptime", 10u32.into()),
            ]),
            rtcp_feedback: vec![],
        };

        let router_options = RouterOptions::new(vec![audio_codec]);
        let router = worker.create_router(router_options).await.unwrap();
        let mut api_key = env::var("API_KEY").unwrap_or(String::from(""));

        if api_key == String::from("") {
            api_key = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect();

            logger::log_info(format!("Auto Generated API_KEY: {}", api_key));
        }

        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            producers: Arc::new(RwLock::new(HashMap::new())),
            webrtc_server: webrtc_server,
            router,
            api_key,
        }
    }

    pub fn get_api_key(&self) -> String {
        self.api_key.clone()
    }

    // Client

    pub async fn send_producers_ids(&self, client_id: usize, transaction_uuid: String) {
        let client_guard = self.clients.read().await;
        let client = client_guard.get(&client_id);

        if client.is_some() {
            let client = client.unwrap().clone();
            drop(client_guard);

            let mut ids: Vec<ProducerId> = vec![];
            for client in self.clients.read().await.values() {
                if let Some(id) = client.get_producer_id() {
                    ids.push(id);
                }
            }

            client.send(WsResponses {
                kind: AddClients { ids },
                transaction_uuid,
            });
        }
    }

    pub async fn connect_client(&self, client: Client) {
        let id = client.get_id();
        logger::log_info(format!("Client #{id} has logged in."));
        self.clients.write().await.insert(id, client);
    }

    pub async fn get_rtp_capabilities(&self) -> RtpCapabilitiesFinalized {
        self.router.rtp_capabilities().clone()
    }

    pub async fn disconnect_client(&self, client_id: usize) {
        let producer_id = self
            .clients
            .read()
            .await
            .get(&client_id)
            .and_then(|c| c.get_producer_id());
        let is_server = self
            .clients
            .read()
            .await
            .get(&client_id)
            .map(|c| c.is_connected_as_server())
            .unwrap_or(false);
        if let Some(producer_id) = producer_id {
            self.producers.write().await.remove(&producer_id);
            for client in self.clients.write().await.values_mut() {
                if client.get_id() != client_id {
                    client.send(WsResponses {
                        kind: RemoveClient { id: producer_id },
                        transaction_uuid: Uuid::new_v4().to_string(),
                    });
                    client.remove_consumer(producer_id);
                }
            }
        }
        self.clients.write().await.remove(&client_id);
        if is_server {
            logger::log_info(format!("Server Client #{client_id} has logged out."));
        } else {
            logger::log_info(format!("Client #{client_id} has logged out."));
        }
    }

    pub async fn client_is_connected(&self, client_id: usize) -> bool {
        let client_guard = self.clients.read().await;
        if let Some(client) = client_guard.get(&client_id) {
            return client.is_connected();
        }
        false
    }

    pub async fn set_code(&self, client_id: usize, code: String) {
        let mut client_guard = self.clients.write().await;
        if let Some(client) = client_guard.get_mut(&client_id) {
            client.code = code.clone();
            client.set_connected();
            logger::log_info(format!(
                "Client #{} has set their code to {}",
                client.get_id(),
                code
            ));
        }
    }

    pub async fn create_transport(
        &self,
        client_id: usize,
        is_producer: bool,
        transaction_uuid: String,
    ) -> Option<WebRtcTransport> {
        if self.client_is_connected(client_id).await {
            let options = WebRtcTransportOptions::new_with_server(self.webrtc_server.clone());
            let transport = self.router.create_webrtc_transport(options).await.unwrap();

            let mut clients_guard = self.clients.write().await;
            let client = clients_guard.get_mut(&client_id);
            if client.is_none() {
                return None;
            }
            let client = client.unwrap();
            client.set_transport(is_producer, transport.clone(), transaction_uuid);
            return Some(transport);
        }
        return None;
    }

    pub async fn connect_transport(
        &self,
        client_id: usize,
        is_producer: bool,
        dtls_parameters: DtlsParameters,
        transaction_uuid: String,
    ) {
        if self.client_is_connected(client_id).await {
            let mut clients_guard = self.clients.write().await;
            let client = clients_guard.get_mut(&client_id);
            if client.is_none() {
                return;
            }
            let client = client.unwrap();
            client
                .connect_transport(is_producer, dtls_parameters, transaction_uuid)
                .await;
        }
    }

    pub async fn produce(
        &self,
        client_id: usize,
        kind: MediaKind,
        rtp_parameters: RtpParameters,
        transaction_uuid: String,
    ) -> Option<Producer> {
        if self.client_is_connected(client_id).await {
            let mut clients_guard = self.clients.write().await;
            let client = clients_guard.get_mut(&client_id);
            if client.is_none() {
                return None;
            }
            let client = client.unwrap();
            let producer = client.produce(kind, rtp_parameters, transaction_uuid).await;
            drop(clients_guard);
            let producer_clone = producer.clone();
            if producer.is_some() {
                let producer_id = producer.unwrap().id();
                let clients_copy = self.clients.read().await.clone();
                self.producers.write().await.insert(producer_id, client_id);
                tokio::spawn(async move {
                    for client in clients_copy.values() {
                        if client.get_id() != client_id {
                            let uuid = Uuid::new_v4().to_string();
                            client.send(WsResponses {
                                kind: AddClients {
                                    ids: vec![producer_id],
                                },
                                transaction_uuid: uuid,
                            });
                        }
                    }
                });
            }
            return producer_clone;
        }

        return None;
    }

    pub async fn consume(
        &self,
        client_id: usize,
        producer_id: ProducerId,
        rtp_capabilities: RtpCapabilities,
        transaction_uuid: String,
    ) -> Option<Consumer> {
        if self.client_is_connected(client_id).await {
            let mut client_guard = self.clients.write().await;
            let client = client_guard.get_mut(&client_id);

            if let Some(client) = client {
                return client
                    .consume(producer_id, rtp_capabilities, transaction_uuid)
                    .await;
            }
        }

        None
    }

    // Server
    pub async fn set_connected_as_server(&self, client_id: usize) {
        let mut client_guard = self.clients.write().await;
        if let Some(client) = client_guard.get_mut(&client_id) {
            client.set_connected_as_server();
            logger::log_info(format!(
                "Client #{} is identified as a server.",
                client.get_id()
            ));
        }
    }

    pub async fn handle_client_as_server(&self, clients: Vec<ClientServerDefinition>) {
        let mut clients_by_code: HashMap<String, Client> = HashMap::new();
        let max_distance: f64 = env::var("MAX_DISTANCE").unwrap_or(String::from("20")).parse().unwrap_or(20.0);

        for client in self.clients.read().await.values() {
            if clients.iter().into_iter().any(|c| c.code == client.code) && client.get_producer_id().is_some() {
                clients_by_code.insert(client.code.clone(), client.clone());
            }
        }

        for client in &clients {
            let mut ws_client = clients_by_code.get(&client.code).unwrap().clone();
            for client_target in &clients {
                let ws_client_target = clients_by_code.get(&client_target.code).unwrap();
                let producer_id: Option<ProducerId> = ws_client_target.get_producer_id();

                if let Some(producer_id) = producer_id {
                    if client.distance_to(client_target) > max_distance {
                        ws_client.mute_consumer(producer_id).await;
                    } else {
                        ws_client.remove_consumer(producer_id);
                    }
                }
            }
        }
    }
}
