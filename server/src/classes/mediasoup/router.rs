use std::{collections::HashMap, num::{NonZeroU8, NonZeroU32}, sync::{Arc, atomic::AtomicUsize}};

use getset::Getters;
use mediasoup::{router::{Router, RouterOptions}, types::rtp_parameters::{MimeTypeAudio, RtpCapabilitiesFinalized, RtpCodecCapability, RtpCodecParametersParameters}, webrtc_transport::{WebRtcTransport, WebRtcTransportOptions}, worker::Worker};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{classes::{client::Client, ws::{requests::ClientServerDefinition, responses::{WsResponses, WsResponsesKind}}}, utils::{config::Config, logger}};

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct InariRouter {
    router: Router,
    name: String,
    permanent: bool,
    max_distance: f64,
    client_count: Arc<AtomicUsize>,
    clients: Arc<RwLock<HashMap<usize, Client>>>
}

impl InariRouter {
    pub async fn new(worker: &Worker, config: &Config, name: String, permanent: bool) -> Self {

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
            max_distance: *config.max_distance(),
            client_count: Arc::new(AtomicUsize::new(0)),
            clients: Arc::new(RwLock::new(HashMap::new()))
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
        self.clients.write().await.remove(client.id());

        if !self.permanent && self.client_count.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            client.bundle().mediasoup_manager.close_router(self.name.clone()).await;
        }

        if let Some(producer) = client.producer() {
            for client in self.clients.read().await.values() {
                let uuid = Uuid::new_v4().to_string();
                client.send(
                    WsResponses {
                        kind: WsResponsesKind::RemoveClient { id: producer.id() },
                        transaction_uuid: uuid
                    }
                );
            }
        }
    }

    pub async fn register_client(&self, client: Client) {
        self.clients.write().await.insert(*client.id(), client);
    }

    pub async fn create_transport(&self, client: &Client) -> WebRtcTransport {
        let options = WebRtcTransportOptions::new_with_server(client.bundle().mediasoup_manager.webrtc_server().clone());
        let transport = self.router.create_webrtc_transport(options).await.unwrap(); 
        transport
    }

    // Sorry for my english, i'm belgian :)
    pub async fn handle_client_as_server(&self, clients: Vec<ClientServerDefinition>) {
        let mut clients_by_code: HashMap<String, Client> = HashMap::new(); // Mapping Code -> Client

        for client in self.clients.read().await.values() { // Iterate over each client.
            // If the client code matches a code used by a connector's client 
            // and if client has producer ("Producer" = Audio stream from the microphone. If there is one, lol.).
            if clients.iter().any(|c| c.code == client.code().clone())
                && client.producer().is_some()
            {
                // Insert map code -> real client (clone why ? Because a lot of elements in client are shared)
                clients_by_code.insert(client.code().clone(), client.clone());
            }
        }

        // for connector's client in connector's payload
        for client in &clients {
            // Get Connected Client from connector's client code. 
            // Continue loop else.
            let Some(ws_client) = clients_by_code.get(&client.code).cloned() else { continue };
            
            // Mute all consumers for the client. ("Consumer" = Audio stream from other clients)
            ws_client.mute_consumers().await;
            
            // for connector's client in connector's payload
            for client_target in &clients {
                if client.code == client_target.code { continue; } // ignored self
                
                // Get client only if in connector's payload
                // continue loop else.
                let Some(ws_client_target) = clients_by_code.get(&client_target.code) else { continue };
                // Get producer if exist, else continue loop. (No producer ? Get lost! )
                let Some(producer) = ws_client_target.producer() else { continue };

                // Get distance between client A and client B or client is force muted by connector
                if client.distance_to(client_target) > self.max_distance || client_target.muted {
                    ws_client.mute_consumer(producer.id()).await; // Mute client B for client A
                } else {
                    ws_client.resume_consumer(producer.id()).await; // UnMute client B for client A
                    
                }
            }
        }
    }
}