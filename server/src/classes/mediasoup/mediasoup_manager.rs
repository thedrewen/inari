use std::net::Ipv4Addr;

use event_listener_primitives::HandlerId;
use mediasoup::{types::data_structures::ListenInfo, webrtc_server::{WebRtcServer, WebRtcServerListenInfos, WebRtcServerOptions}, worker::WorkerSettings, worker_manager::WorkerManager};

use crate::utils::{config::Config, logger};

#[derive(Clone)]
pub struct MediaSoupManager {
    webrtc_server: WebRtcServer,

    
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

        logger::log_info(format!("WebRtcServer is running..."));
        Self {
            webrtc_server,
            _webrtc_server_on_close
        }
    }
}