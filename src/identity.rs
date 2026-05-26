use crate::error::{P2pError, Result};
use libp2p::identity::Keypair;
use zeroize::Zeroizing;

/// Loads a keypair from path, or generates a fresh Ed25519 keypair and
/// persists it there.  The key bytes are zeroised from memory on drop.
pub fn load_or_create_keypair(path: &str) -> Result<Keypair> {
    let p = std::path::Path::new(path);

    if p.exists() {
        let raw = Zeroizing::new(std::fs::read(p)?);
        return Keypair::from_protobuf_encoding(&raw)
            .map_err(|e| P2pError::Identity(e.to_string()));
    }

    // Create parent directories if needed (e.g. app data dir on mobile).
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let keypair = Keypair::generate_ed25519();
    let encoded = Zeroizing::new(
        keypair
            .to_protobuf_encoding()
            .map_err(|e| P2pError::Identity(e.to_string()))?,
    );
    std::fs::write(p, encoded.as_slice())?;
    log::info!("New identity generated → {path}");

    Ok(keypair)
}