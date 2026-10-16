use serde::{Serialize, Serializer};

#[derive(Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub from_peer: String,
    pub topic: String,
    #[serde(serialize_with = "serialize_as_base64")]
    pub ciphertext: Vec<u8>,
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

fn serialize_as_base64<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    serializer.serialize_str(&encoded)
}

#[derive(Serialize)]
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

pub enum NodeCommand {
    Subscribe(String),
    Publish { topic: String, data: Vec<u8> },
    Dial(String),
    Shutdown,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}