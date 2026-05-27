#![cfg(target_arch = "wasm32")]

use futures::channel::mpsc;
use futures::StreamExt;
use libp2p::{
    gossipsub, identify,
    swarm::SwarmEvent,
    Multiaddr, Swarm,
};
use libp2p_webrtc_websys as webrtc_websys;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static NODE: RefCell<Option<NodeState>> = RefCell::new(None);
}

struct NodeState {
    cmd_tx: mpsc::UnboundedSender<Command>,
    ev_rx: mpsc::UnboundedReceiver<Event>,
}

enum Command {
    Subscribe(String),
    Publish { topic: String, data: Vec<u8> },
    Shutdown,
}

enum Event {
    Message { topic: String, from: String, text: String },
    Connected(String),
    Disconnected(String),
}

#[wasm_bindgen]
pub async fn start_node(desktop_webrtc_addr: String) -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Info)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let (cmd_tx, cmd_rx) = mpsc::unbounded();
    let (ev_tx, ev_rx) = mpsc::unbounded();

    NODE.with(|cell| {
        *cell.borrow_mut() = Some(NodeState { cmd_tx: cmd_tx.clone(), ev_rx });
    });

    spawn_local(run_swarm(desktop_webrtc_addr, cmd_rx, ev_tx));
    Ok(())
}

#[wasm_bindgen]
pub fn subscribe(topic: String) {
    NODE.with(|cell| {
        if let Some(ref state) = *cell.borrow() {
            let _ = state.cmd_tx.unbounded_send(Command::Subscribe(topic));
        }
    });
}

#[wasm_bindgen]
pub fn send_message(topic: String, text: String) {
    NODE.with(|cell| {
        if let Some(ref state) = *cell.borrow() {
            let _ = state.cmd_tx.unbounded_send(Command::Publish {
                topic,
                data: text.into_bytes(),
            });
        }
    });
}

#[wasm_bindgen]
pub fn poll_event() -> Option<String> {
    NODE.with(|cell| {
        if let Some(ref mut state) = *cell.borrow_mut() {
            match state.ev_rx.try_next() {
                Ok(Some(ev)) => {
                    let json = match ev {
                        Event::Message { topic, from, text } => {
                            serde_json::json!({
                                "type": "message",
                                "topic": topic,
                                "from": from,
                                "text": text,
                            })
                            .to_string()
                        }
                        Event::Connected(peer) => {
                            serde_json::json!({
                                "type": "connected",
                                "peer": peer,
                            })
                            .to_string()
                        }
                        Event::Disconnected(peer) => {
                            serde_json::json!({
                                "type": "disconnected",
                                "peer": peer,
                            })
                            .to_string()
                        }
                    };
                    Some(json)
                }
                _ => None,
            }
        } else {
            None
        }
    })
}

#[wasm_bindgen]
pub fn shutdown() {
    NODE.with(|cell| {
        if let Some(ref state) = *cell.borrow() {
            let _ = state.cmd_tx.unbounded_send(Command::Shutdown);
        }
        *cell.borrow_mut() = None;
    });
}

async fn run_swarm(
    desktop_addr: String,
    mut cmd_rx: mpsc::UnboundedReceiver<Command>,
    ev_tx: mpsc::UnboundedSender<Event>,
) {
    let mut swarm: Swarm<Behaviour> = libp2p::SwarmBuilder::with_new_identity()
        .with_wasm_bindgen()
        .with_other_transport(|key| {
            Ok(webrtc_websys::Transport::new(webrtc_websys::Config::new(&key)))
        })
        .expect("transport")
        .with_behaviour(|key| {
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub::ConfigBuilder::default()
                    .build()
                    .expect("gossipsub config"),
            )
            .expect("gossipsub init");

            let identify = identify::Behaviour::new(identify::Config::new(
                "/p2p-chat/1.0.0".to_string(),
                key.public(),
            ));

            Ok(Behaviour { gossipsub, identify })
        })
        .expect("behaviour")
        .with_swarm_config(|c| {
            c.with_idle_connection_timeout(std::time::Duration::from_secs(60))
        })
        .build();

    let addr: Multiaddr = desktop_addr.parse().expect("invalid multiaddr");
    log::info!("Dialing {addr}");
    if let Err(e) = swarm.dial(addr) {
        log::error!("dial error: {e}");
        let _ = ev_tx.unbounded_send(Event::Disconnected("dial failed".into()));
        return;
    }

    loop {
        futures::select! {
            cmd = cmd_rx.next() => {
                match cmd {
                    Some(Command::Subscribe(topic)) => {
                        let t = gossipsub::IdentTopic::new(&topic);
                        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&t) {
                            log::error!("subscribe error: {e}");
                        }
                    }
                    Some(Command::Publish { topic, data }) => {
                        let t = gossipsub::IdentTopic::new(&topic);
                        if let Err(e) = swarm.behaviour_mut().gossipsub.publish(t, data) {
                            log::error!("publish error: {e}");
                        }
                    }
                    Some(Command::Shutdown) | None => break,
                }
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        log::info!("connected: {peer_id}");
                        let _ = ev_tx.unbounded_send(Event::Connected(peer_id.to_string()));
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        log::info!("disconnected: {peer_id}");
                        let _ = ev_tx.unbounded_send(Event::Disconnected(peer_id.to_string()));
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { message, propagation_source, .. }
                    )) => {
                        let _ = ev_tx.unbounded_send(Event::Message {
                            topic: message.topic.to_string(),
                            from: propagation_source.to_string(),
                            text: String::from_utf8_lossy(&message.data).to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

use libp2p::swarm::NetworkBehaviour;

#[derive(NetworkBehaviour)]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
}