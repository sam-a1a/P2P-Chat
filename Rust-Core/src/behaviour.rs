use libp2p::gossipsub;
use libp2p::identify;
use libp2p::mdns;
use libp2p::swarm::NetworkBehaviour;

#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
}