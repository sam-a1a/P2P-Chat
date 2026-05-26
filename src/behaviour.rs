use libp2p::{gossipsub, identify, mdns, swarm::NetworkBehaviour};

/// The combined network behaviour for the LAN chat node.
///
/// #[derive(NetworkBehaviour)] automatically generates:
///   wiring between the sub-behaviours and the Swarm
///   the ChatBehaviourEvent enum (one variant per field)
///   From<SubEvent> for ChatBehaviourEvent impls
///
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ChatBehaviourEvent")]
pub struct ChatBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns:      mdns::tokio::Behaviour,
    pub identify:  identify::Behaviour,
}

// Event enum (explicit so node.rs can match ergonomically)

#[derive(Debug)]
pub enum ChatBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Identify(identify::Event),
}

impl From<gossipsub::Event> for ChatBehaviourEvent {
    fn from(e: gossipsub::Event) -> Self { Self::Gossipsub(e) }
}
impl From<mdns::Event> for ChatBehaviourEvent {
    fn from(e: mdns::Event) -> Self { Self::Mdns(e) }
}
impl From<identify::Event> for ChatBehaviourEvent {
    fn from(e: identify::Event) -> Self { Self::Identify(e) }
}