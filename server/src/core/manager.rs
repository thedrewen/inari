use std::{collections::HashMap, env, net::Ipv4Addr, num::{NonZeroU8, NonZeroU32}, sync::Arc};

use mediasoup::{
    producer::Producer, router::{Router, RouterOptions}, types::{data_structures::{DtlsParameters, ListenInfo}, rtp_parameters::{MediaKind, MimeTypeAudio, RtpCodecCapability, RtpCodecParametersParameters, RtpParameters}}, webrtc_server::{WebRtcServer, WebRtcServerListenInfos, WebRtcServerOptions}, webrtc_transport::{WebRtcTransport, WebRtcTransportOptions}, worker::WorkerSettings, worker_manager::WorkerManager,
};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{core::{client::Client, responses::{WsResponses, WsResponsesKind::{AddClients, RemoveClient}}}, logger};

#[derive(Clone)]
pub struct Manager {
    clients: Arc<RwLock<HashMap<usize, Client>>>,
    router: Router,
    webrtc_server: WebRtcServer,

    
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

        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            webrtc_server: webrtc_server,
            router
        }
    }




    pub async fn connect_client(&self, client: Client) {
        let id = client.get_id();
        logger::log_info(format!("User #{id} has logged in."));

        let rtp_capabilities = self.router.rtp_capabilities();
        let uuid = Uuid::new_v4().to_string();

        let response = WsResponses {
            kind: super::responses::WsResponsesKind::RtpCapabilities { rtp_capabilities },
            transaction_uuid: uuid
        };
        
        client.send(response);

        self.clients.write().await.insert(id, client);
    }

    pub async fn disconnect_client(&self, user_id: usize) {

        let producer_id = self.clients.read().await.get(&user_id).and_then(|c| c.get_producer_id());
        if let Some(producer_id) = producer_id {
            for client in self.clients.read().await.values() {
                if client.get_id() != user_id {
                    client.send(WsResponses { kind: RemoveClient { id: producer_id }, transaction_uuid: Uuid::new_v4().to_string() });
                }
            }
        }

        self.clients.write().await.remove(&user_id);
        logger::log_info(format!("User #{user_id} has logged out."));
    }

    pub async fn client_is_connected(&self, user_id: usize) -> bool {
        let client_guard = self.clients.read().await;
        if let Some(client) = client_guard.get(&user_id) {
            return client.is_connected();
        }
        false
    }


    
    pub async fn set_code(&self, user_id: usize, code: String) {
        let mut client_guard = self.clients.write().await;
        let client = client_guard.get_mut(&user_id);
        if client.is_some() {
            let client = client.unwrap();
            client.code = code.clone();
            client.set_connected();
            
            logger::log_info(format!("Client #{} has set their code to {}", client.get_id(), code));
        }
    }

    pub async fn create_transport(&self, user_id: usize, is_producer: bool, transaction_uuid: String) -> Option<WebRtcTransport> {
        if self.client_is_connected(user_id).await {
            let options = WebRtcTransportOptions::new_with_server(self.webrtc_server.clone());
            let transport = self.router.create_webrtc_transport(options).await.unwrap();

            let mut users_guard = self.clients.write().await;
            let user = users_guard.get_mut(&user_id);
            if user.is_none() {
                return None;
            }
            let user = user.unwrap();
            user.set_transport(is_producer, transport.clone(), transaction_uuid);
            return Some(transport);
        }
        return None;
    }

    pub async fn connect_transport(&self, user_id: usize, is_producer: bool, dtls_parameters: DtlsParameters, transaction_uuid: String) {
        if self.client_is_connected(user_id).await {
            let mut users_guard = self.clients.write().await;
            let user = users_guard.get_mut(&user_id);
            if user.is_none() {
                return;
            }
            let user = user.unwrap();
            user.connect_transport(is_producer, dtls_parameters, transaction_uuid).await;
        }
    }

    pub async fn produce(&self, user_id: usize, kind: MediaKind, rtp_parameters: RtpParameters, transaction_uuid: String) -> Option<Producer> {
        if self.client_is_connected(user_id).await {
            let mut users_guard = self.clients.write().await;
            let user = users_guard.get_mut(&user_id);
            if user.is_none() {
                return None;
            }
            let user = user.unwrap();
            let producer = user.produce(kind, rtp_parameters, transaction_uuid).await;
            drop(users_guard);
            let producer_clone = producer.clone();
            if producer.is_some() {
                let producer_id = producer.unwrap().id();
                let clients_copy = self.clients.read().await.clone();

                tokio::spawn(async move {
                    for client in clients_copy.values() {
                        if client.get_id() != user_id {
                            let uuid = Uuid::new_v4().to_string();
                            client.send(WsResponses { kind: AddClients { ids: vec![producer_id] }, transaction_uuid: uuid });
                        }
                    }
                });
            }
            return producer_clone;
        }

        return None;
    }
}
