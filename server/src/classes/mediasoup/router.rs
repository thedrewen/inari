use std::{num::{NonZeroU8, NonZeroU32}, sync::{Arc, atomic::AtomicUsize}};

use mediasoup::{router::{Router, RouterOptions}, types::rtp_parameters::{MimeTypeAudio, RtpCapabilitiesFinalized, RtpCodecCapability, RtpCodecParametersParameters}, worker::Worker};

use crate::{classes::client::Client, utils::logger};

#[derive(Clone)]
pub struct InariRouter {
    router: Router,
    name: String,
    permanent: bool,
    client_count: Arc<AtomicUsize>
}

impl InariRouter {
    pub async fn new(worker: &Worker, name: String, permanent: bool) -> Self {

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

        logger::log_info(format!("Router {} running...", name));

        Self {
            router,
            name,
            permanent,
            client_count: Arc::new(AtomicUsize::new(0))
        }
    }

    pub fn get_rtp_capabilities(&self) -> RtpCapabilitiesFinalized {
        self.router.rtp_capabilities().clone()
    }

    pub fn add_client(&self, client: &Client) {
        logger::log_info(format!("Client #{} has logged in {}", client.id(), self.name));
        self.client_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn remove_client(&self, client: &Client) {
        logger::log_info(format!("Client #{} has logged out {}", client.id(), self.name));
        self.client_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        if !self.permanent && self.client_count.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            client.bundle().mediasoup_manager.close_router(self.name.clone()).await;
        }
    }
}