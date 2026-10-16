use crate::error::{P2pError, Result};
use libp2p::identity::Keypair;
use zeroize::Zeroizing;

/// Loads a keypair from path, or generates a fresh Ed25519 keypair and
/// persists it there. The key bytes are zeroised from memory on drop.
pub fn load_or_create_keypair(path: &str) -> Result<Keypair> {
    let p = std::path::Path::new(path);

    if p.exists() {
        let raw = Zeroizing::new(std::fs::read(p)?);
        return Keypair::from_protobuf_encoding(&raw)
            .map_err(|e| P2pError::Identity(e.to_string()));
    }

    // Create parent directories if needed (e.g. app data dir on mobile).
    if let Some(parent) = p.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }

    let keypair = Keypair::generate_ed25519();
    let encoded = Zeroizing::new(
        keypair
            .to_protobuf_encoding()
            .map_err(|e| P2pError::Identity(e.to_string()))?,
    );
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(p)?;
    use std::io::Write;
    file.write_all(encoded.as_slice())?;
    file.sync_all()?;
    log::info!("New identity generated → {path}");

    Ok(keypair)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unique_persistent_private_identities() {
        let dir = std::env::temp_dir().join(format!("p2p-key-{}", uuid::Uuid::new_v4()));
        let a_path = dir.join("a/identity.key");
        let b_path = dir.join("b/identity.key");
        let a = load_or_create_keypair(a_path.to_str().unwrap()).unwrap().public().to_peer_id();
        let b = load_or_create_keypair(b_path.to_str().unwrap()).unwrap().public().to_peer_id();
        assert_ne!(a, b);
        assert_eq!(a, load_or_create_keypair(a_path.to_str().unwrap()).unwrap().public().to_peer_id());
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&a_path).unwrap().permissions().mode() & 0o777, 0o600);
        }
        std::fs::write(&a_path, b"corrupt").unwrap();
        assert!(load_or_create_keypair(a_path.to_str().unwrap()).is_err());
        assert_eq!(std::fs::read(&a_path).unwrap(), b"corrupt");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
