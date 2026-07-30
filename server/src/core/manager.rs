use std::{collections::HashMap, env, net::Ipv4Addr, num::{NonZeroU8, NonZeroU32}, sync::Arc};

use mediasoup::{
    router::{Router, RouterOptions}, types::{data_structures::ListenInfo, rtp_parameters::{MimeTypeAudio, RtpCodecCapability, RtpCodecParametersParameters}}, webrtc_server::{WebRtcServer, WebRtcServerListenInfos, WebRtcServerOptions}, worker::WorkerSettings, worker_manager::WorkerManager,
};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{core::{client::Client, responses::WsResponses}, logger};

#[derive(Clone)]
pub struct Manager {
    clients: Arc<RwLock<HashMap<usize, Client>>>,
    router: Router,
    _webrtc_server: WebRtcServer
}

impl Manager {
    pub async fn new() -> Self {
        let w_manager = WorkerManager::new();
        let worker = w_manager
            .create_worker(WorkerSettings::default())
            .await
            .unwrap();

        let listen_info = ListenInfo {
            protocol: mediasoup::types::data_structures::Protocol::Udp,
            ip: std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            announced_address: Some(
                env::var("SERVER_PUBLIC_IP").unwrap_or(String::from("127.0.0.1")),
            ),
            port: Some(44444),
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
            _webrtc_server: webrtc_server,
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

    pub async fn disconnect_client(&self, id: usize) {
        self.clients.write().await.remove(&id);
        logger::log_info(format!("User #{id} has logged out."));
    }
}
