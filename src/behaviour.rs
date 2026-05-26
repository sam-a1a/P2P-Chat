use libp2p::{gossipsub, identify, mdns, swarm::NetworkBehaviour};

#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns:      mdns::tokio::Behaviour,
    pub identify:  identify::Behaviour,
}