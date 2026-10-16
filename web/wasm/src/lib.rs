#![cfg(target_arch = "wasm32")]
use futures::{StreamExt, channel::mpsc};
use libp2p::{Multiaddr, Swarm, Transport, gossipsub, identify, noise, swarm::SwarmEvent, yamux};
use serde::Serialize;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

thread_local! { static NODE: RefCell<Option<NodeState>> = const { RefCell::new(None) }; }
struct NodeState {
    commands: mpsc::UnboundedSender<Command>,
    events: mpsc::UnboundedReceiver<Event>,
}
enum Command {
    Publish(String, String),
    Shutdown,
}
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Event {
    Message { from: String, text: String },
    Connected { peer: String },
    Disconnected { peer: String },
    Published { id: String, text: String },
    Error { message: String },
}
fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[wasm_bindgen]
pub fn start_node(address: String) -> Result<(), JsValue> {
    let _ = console_log::init_with_level(log::Level::Info);
    if NODE.with(|node| node.borrow().is_some()) {
        return Err(js_error("Node already running"));
    }
    let address: Multiaddr = address.parse().map_err(js_error)?;
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_wasm_bindgen()
        .with_other_transport(|key| {
            Ok(libp2p::websocket_websys::Transport::default()
                .upgrade(libp2p::core::upgrade::Version::V1Lazy)
                .authenticate(noise::Config::new(key)?)
                .multiplex(yamux::Config::default()))
        })
        .map_err(|e| js_error(e))?
        .with_behaviour(|key| {
            let config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(std::time::Duration::from_secs(1))
                .build()
                .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e))?;
            let mut gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                config,
            )?;
            gossipsub.subscribe(&gossipsub::IdentTopic::new("chat"))?;
            Ok(Behaviour {
                gossipsub,
                identify: identify::Behaviour::new(identify::Config::new(
                    "/p2p-chat/1.0.0".into(),
                    key.public(),
                )),
            })
        })
        .map_err(js_error)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(300)))
        .build();
    swarm.dial(address).map_err(js_error)?;
    let (commands, rx) = mpsc::unbounded();
    let (tx, events) = mpsc::unbounded();
    NODE.with(|node| *node.borrow_mut() = Some(NodeState { commands, events }));
    spawn_local(run(swarm, rx, tx));
    Ok(())
}
#[wasm_bindgen]
pub fn send_message(id: String, text: String) -> Result<(), JsValue> {
    if text.trim().is_empty() || text.len() > 65536 {
        return Err(js_error("Message must contain 1–65536 UTF-8 bytes"));
    }
    NODE.with(|node| {
        node.borrow()
            .as_ref()
            .ok_or_else(|| js_error("Not connected"))?
            .commands
            .unbounded_send(Command::Publish(id, text))
            .map_err(js_error)
    })
}
#[wasm_bindgen]
pub fn poll_event() -> Option<String> {
    NODE.with(|node| {
        node.borrow_mut()
            .as_mut()?
            .events
            .try_recv()
            .ok()
            .and_then(|e| serde_json::to_string(&e).ok())
    })
}
#[wasm_bindgen]
pub fn shutdown() {
    NODE.with(|node| {
        if let Some(state) = node.borrow_mut().take() {
            let _ = state.commands.unbounded_send(Command::Shutdown);
        }
    });
}
async fn run(
    mut swarm: Swarm<Behaviour>,
    mut commands: mpsc::UnboundedReceiver<Command>,
    events: mpsc::UnboundedSender<Event>,
) {
    loop {
        futures::select! {
            command = commands.next() => match command {
                Some(Command::Publish(id, text)) => {
                    let event = match swarm.behaviour_mut().gossipsub.publish(gossipsub::IdentTopic::new("chat"), text.as_bytes()) {
                        Ok(_) => Event::Published { id, text },
                        Err(e) => Event::Error { message: format!("Message not sent: {e}") },
                    };
                    let _ = events.unbounded_send(event);
                }
                Some(Command::Shutdown) | None => break,
            },
            event = swarm.select_next_some() => {
                let event = match event {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        None
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Subscribed { peer_id, topic }))
                        if topic == gossipsub::IdentTopic::new("chat").hash() => Some(Event::Connected { peer: peer_id.to_string() }),
                    SwarmEvent::ConnectionClosed { peer_id, num_established: 0, .. } => Some(Event::Disconnected { peer: peer_id.to_string() }),
                    SwarmEvent::OutgoingConnectionError { error, .. } => Some(Event::Error { message: format!("Connection failed: {error}") }),
                    SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message { message, propagation_source, .. })) => Some(Event::Message {
                        from: message.source.unwrap_or(propagation_source).to_string(), text: String::from_utf8_lossy(&message.data).into_owned(),
                    }),
                    _ => None,
                };
                if let Some(event) = event { let _ = events.unbounded_send(event); }
            }
        }
    }
}
#[derive(libp2p::swarm::NetworkBehaviour)]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
}
