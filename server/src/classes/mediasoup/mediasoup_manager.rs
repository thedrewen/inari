use std::{collections::HashMap, net::Ipv4Addr, sync::Arc};

use event_listener_primitives::HandlerId;
use mediasoup::{router, types::data_structures::ListenInfo, webrtc_server::{WebRtcServer, WebRtcServerListenInfos, WebRtcServerOptions}, worker::{Worker, WorkerSettings}, worker_manager::WorkerManager};
use tokio::sync::RwLock;

use crate::{classes::mediasoup::router::InariRouter, utils::{config::Config, logger}};

#[derive(Clone)]
pub struct MediaSoupManager {
    webrtc_server: WebRtcServer,
    worker: Worker,
    routers: Arc<RwLock<HashMap<String, InariRouter>>>,
    
    _webrtc_server_on_close: HandlerId
}

impl MediaSoupManager {
    pub async fn new(config: &Config) -> Self {
        let w_manager = WorkerManager::new();
        let worker = w_manager
            .create_worker(WorkerSettings::default())
            .await
            .unwrap();

        let listen_info = ListenInfo {
            protocol: mediasoup::types::data_structures::Protocol::Udp,
            ip: std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            announced_address: Some(
                config.server_public_ip().clone(),
            ),
            port: Some(config.webrtc_server_port().clone()),
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

        let _webrtc_server_on_close = webrtc_server.on_close(|| {
            logger::log_error("WebrtcServer Closed !".into());
        });

        let default_router = InariRouter::new(&worker, "default".into(), true).await;
        let mut routers = HashMap::new();
        routers.insert("default".into(), default_router);

        logger::log_info(format!("WebRtcServer is running..."));
        Self {
            webrtc_server,
            worker,
            _webrtc_server_on_close,
            routers: Arc::new(RwLock::new(routers))
        }
    }


    pub async fn get_router(&self, name: String) -> InariRouter {
        if let Some(router) = self.routers.read().await.get(&name) {
            return router.clone();
        } 

        let router = InariRouter::new(&self.worker, name.clone(), false).await;

        self.routers.write().await.insert(name, router.clone());

        router
    }

    pub async fn close_router(&self, name: String) {
        logger::log_info(format!("Router {} closed !", name));
        self.routers.write().await.remove(&name);
    }
}