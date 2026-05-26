use serde::{Deserialize, Serialize};

// Wire message

/// A chat message as it travels over Gossipsub.
/// `ciphertext` is a Megolm-encrypted payload (see crypto module).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub from_peer: String,   // PeerId as string
    pub topic: String,       // Gossipsub topic name
    pub ciphertext: Vec<u8>, // Megolm encrypted content
    pub timestamp_secs: u64,
}

impl ChatMessage {
    pub fn new(from_peer: &str, topic: impl Into<String>, ciphertext: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_peer: from_peer.to_owned(),
            topic: topic.into(),
            ciphertext,
            timestamp_secs: now_secs(),
        }
    }
}

// Plain (pre-encryption) message

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlainMessage {
    pub text: String,
}

// Events emitted by the node to the application layer
// All libp2p types (PeerId, Multiaddr) are converted to strings here so that
// the FFI layer can serialise them to JSON with no extra dependencies.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeEvent {
    PeerDiscovered {
        peer: String,
    },
    PeerExpired {
        peer: String,
    },
    ConnectionEstablished {
        peer: String,
        address: String,
    },
    ConnectionClosed {
        peer: String,
    },
    MessageReceived(ChatMessage),
    ListeningOn {
        address: String,
    },
    Error {
        message: String,
    },
}

// Commands sent from the application layer into the node

#[derive(Debug)]
pub enum NodeCommand {
    Subscribe(String),
    Publish { topic: String, data: Vec<u8> },
    Shutdown,
}

// Helpers

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}