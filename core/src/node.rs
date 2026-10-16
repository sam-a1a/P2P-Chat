use crate::behaviour::ChatBehaviourEvent;
use crate::{
    behaviour::ChatBehaviour,
    error::{P2pError, Result},
    types::{ChatMessage, NodeCommand, NodeEvent},
};
use futures::StreamExt;
use libp2p::mdns;
use libp2p::{Swarm, Transport, gossipsub, identify, noise, swarm::SwarmEvent, tcp, yamux};
use std::time::Duration;
use tokio::sync::{mpsc, watch};

pub struct NodeConfig {
    pub tcp_address: String,
    pub websocket_address: Option<String>,
}
impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            tcp_address: "/ip4/0.0.0.0/tcp/53493".into(),
            websocket_address: Some("/ip4/0.0.0.0/tcp/9090/ws".into()),
        }
    }
}
const PROTOCOL_VERSION: &str = "/p2p-chat/1.0.0";

pub struct NodeHandle {
    pub command_tx: mpsc::UnboundedSender<NodeCommand>,
    pub event_rx: mpsc::UnboundedReceiver<NodeEvent>,
    shutdown_tx: watch::Sender<bool>,
}

impl NodeHandle {
    pub fn subscribe(&self, topic: &str) {
        let _ = self
            .command_tx
            .send(NodeCommand::Subscribe(topic.to_owned()));
    }

    pub fn publish(&self, topic: &str, data: Vec<u8>) {
        let _ = self.command_tx.send(NodeCommand::Publish {
            topic: topic.to_owned(),
            data,
        });
    }

    pub fn dial(&self, addr: &str) {
        let _ = self.command_tx.send(NodeCommand::Dial(addr.to_owned()));
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

struct Node {
    swarm: Swarm<ChatBehaviour>,
    event_tx: mpsc::UnboundedSender<NodeEvent>,
    command_rx: mpsc::UnboundedReceiver<NodeCommand>,
    shutdown_rx: watch::Receiver<bool>,
    config: NodeConfig,
}

impl Node {
    pub async fn run(mut self) {
        let addresses = std::iter::once(self.config.tcp_address.clone())
            .chain(self.config.websocket_address.clone());
        for address in addresses {
            let result = address
                .parse::<libp2p::Multiaddr>()
                .map_err(|e| e.to_string())
                .and_then(|a| self.swarm.listen_on(a).map_err(|e| e.to_string()));
            if let Err(e) = result {
                self.error(format!("listen failed: {e}"));
                return;
            }
        }

        loop {
            tokio::select! {
                biased;
                Ok(_) = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        log::info!("node: shutdown signal received");
                        break;
                    }
                }
                cmd = self.command_rx.recv() => {
                    match cmd {
                        Some(NodeCommand::Subscribe(topic)) => {
                            self.subscribe(&topic);
                        }
                        Some(NodeCommand::Publish { topic, data }) => {
                            self.publish(&topic, data);
                        }
                        Some(NodeCommand::Dial(addr)) => {
                            self.dial_peer(&addr);
                        }
                        Some(NodeCommand::Shutdown) | None => {
                            log::info!("node: command channel closed");
                            break;
                        }
                    }
                }
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event);
                }
            }
        }

        log::info!("node: event loop exited");
    }

    fn error(&self, message: String) {
        let _ = self.event_tx.send(NodeEvent::Error { message });
    }

    fn dial_peer(&mut self, addr_str: &str) {
        match addr_str.parse::<libp2p::Multiaddr>() {
            Ok(ma) => {
                log::info!("dialing {}", addr_str);
                if let Err(e) = self.swarm.dial(ma) {
                    self.error(format!("dial: {e}"));
                }
            }
            Err(e) => self.error(format!("invalid address: {e}")),
        }
    }

    fn subscribe(&mut self, topic_name: &str) {
        let topic = gossipsub::IdentTopic::new(topic_name);
        match self.swarm.behaviour_mut().gossipsub.subscribe(&topic) {
            Ok(true) => log::info!("subscribed to topic '{topic_name}'"),
            Ok(false) => log::debug!("already subscribed to '{topic_name}'"),
            Err(e) => log::error!("subscribe error: {e}"),
        }
    }

    fn publish(&mut self, topic_name: &str, data: Vec<u8>) {
        if data.is_empty() || data.len() > 65536 {
            self.error("Message must contain 1–65536 bytes".into());
            return;
        }
        let text = String::from_utf8_lossy(&data).into_owned();
        let topic = gossipsub::IdentTopic::new(topic_name);
        match self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
            Ok(_) => {
                let _ = self.event_tx.send(NodeEvent::MessagePublished { text });
            }
            Err(e) => {
                log::error!("publish error: {e}");
                let _ = self.event_tx.send(NodeEvent::Error {
                    message: format!("publish: {e}"),
                });
            }
        }
    }

    fn handle_swarm_event(&mut self, event: SwarmEvent<ChatBehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("listening on {address}");
                let _ = self.event_tx.send(NodeEvent::ListeningOn {
                    address: address.to_string(),
                });
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                log::info!("connected: {peer_id}");
                self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                let _ = self.event_tx.send(NodeEvent::ConnectionEstablished {
                    peer: peer_id.to_string(),
                    address: endpoint.get_remote_address().to_string(),
                });
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                num_established: 0,
                ..
            } => {
                log::info!("disconnected: {peer_id}");
                let _ = self.event_tx.send(NodeEvent::ConnectionClosed {
                    peer: peer_id.to_string(),
                });
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                self.error(format!("connection: {error}"))
            }
            SwarmEvent::ListenerError { error, .. } => self.error(format!("listener: {error}")),
            SwarmEvent::Behaviour(bev) => self.handle_behaviour_event(bev),
            _ => {}
        }
    }

    fn handle_behaviour_event(&mut self, event: ChatBehaviourEvent) {
        match event {
            ChatBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                for (peer_id, addr) in peers {
                    log::info!("mDNS discovered {peer_id} @ {addr}");
                    self.swarm.add_peer_address(peer_id, addr.clone());
                    if !self.swarm.is_connected(&peer_id) {
                        let _ = self.swarm.dial(addr);
                    }
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                    let _ = self.event_tx.send(NodeEvent::PeerDiscovered {
                        peer: peer_id.to_string(),
                    });
                }
            }
            ChatBehaviourEvent::Mdns(mdns::Event::Expired(peers)) => {
                for (peer_id, _addr) in peers {
                    log::info!("mDNS expired {peer_id}");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                    let _ = self.event_tx.send(NodeEvent::PeerExpired {
                        peer: peer_id.to_string(),
                    });
                }
            }
            ChatBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            }) => {
                let msg = ChatMessage::new(
                    &message.source.unwrap_or(propagation_source).to_string(),
                    message.topic.as_str(),
                    message.data,
                );
                let _ = self.event_tx.send(NodeEvent::MessageReceived(msg));
            }
            ChatBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed { peer_id, topic }) => {
                log::debug!("{peer_id} subscribed to {topic}");
                if topic == gossipsub::IdentTopic::new("chat").hash() {
                    let _ = self.event_tx.send(NodeEvent::PeerReady { peer: peer_id.to_string() });
                }
            }
            ChatBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. }) => {
                log::debug!(
                    "identified {peer_id}: agent={} protos={:?}",
                    info.agent_version,
                    info.protocols
                );
            }
            _ => {}
        }
    }
}

pub fn start_node(keypair: libp2p::identity::Keypair) -> Result<NodeHandle> {
    start_node_with_config(keypair, NodeConfig::default())
}

pub fn start_node_with_config(
    keypair: libp2p::identity::Keypair,
    config: NodeConfig,
) -> Result<NodeHandle> {
    let swarm = build_swarm(keypair)?;

    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (command_tx, command_rx) = mpsc::unbounded_channel();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let node = Node {
        swarm,
        event_tx,
        command_rx,
        shutdown_rx,
        config,
    };

    tokio::spawn(node.run());

    Ok(NodeHandle {
        command_tx,
        event_rx,
        shutdown_tx,
    })
}

fn build_swarm(keypair: libp2p::identity::Keypair) -> Result<Swarm<ChatBehaviour>> {
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| P2pError::Transport(e.to_string()))?
        .with_other_transport(|key| {
            Ok(
                libp2p::websocket::Config::new(tcp::tokio::Transport::new(tcp::Config::default()))
                    .upgrade(libp2p::core::upgrade::Version::V1Lazy)
                    .authenticate(noise::Config::new(key)?)
                    .multiplex(yamux::Config::default()),
            )
        })
        .map_err(|e| P2pError::Transport(e.to_string()))?
        .with_behaviour(|key| {
            let gossipsub_cfg = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(1))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .build()
                .map_err(|e| anyhow::anyhow!("gossipsub config: {e}"))?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_cfg,
            )
            .map_err(|e| anyhow::anyhow!("gossipsub: {e}"))?;

            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())
                    .map_err(|e| anyhow::anyhow!("mdns: {e}"))?;

            let identify = identify::Behaviour::new(identify::Config::new(
                PROTOCOL_VERSION.to_string(),
                key.public(),
            ));

            Ok(ChatBehaviour {
                gossipsub,
                mdns,
                identify,
            })
        })
        .map_err(|e| P2pError::Behaviour(e.to_string()))?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    Ok(swarm)
}
