#[cfg(not(target_os = "android"))]
use libp2p::mdns;
use libp2p::{gossipsub, identify, swarm::NetworkBehaviour};

#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    #[cfg(not(target_os = "android"))]
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
}