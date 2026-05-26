use crate::error::{P2pError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vodozemac::{
    megolm::{
        GroupSession,
        InboundGroupSession,
        MegolmMessage,
        SessionConfig,
        SessionKey,
    },
    olm::{
        Account,
        OlmMessage,
        Session,
        SessionConfig as OlmSessionConfig,
    },
    Curve25519PublicKey,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyBundle {
    pub curve25519: String,
    pub one_time_keys: Vec<String>,
}

pub struct CryptoEngine {
    account: Account,
    olm: HashMap<String, Session>,
    outbound: HashMap<String, GroupSession>,
    inbound: HashMap<String, InboundGroupSession>,
}

impl CryptoEngine {
    pub fn new() -> Self {
        let mut account = Account::new();
        account.generate_one_time_keys(20);
        Self {
            account,
            olm: HashMap::new(),
            outbound: HashMap::new(),
            inbound: HashMap::new(),
        }
    }

    pub fn public_bundle(&mut self) -> PublicKeyBundle {
        let curve25519 = self.account.curve25519_key().to_base64();
        let one_time_keys: Vec<String> = self
            .account
            .one_time_keys()
            .values()
            .map(|k| k.to_base64())
            .collect();
        self.account.mark_keys_as_published();
        if one_time_keys.len() < 5 {
            self.account.generate_one_time_keys(20);
        }
        PublicKeyBundle {
            curve25519,
            one_time_keys,
        }
    }

    pub fn create_outbound_olm(
        &mut self,
        their_curve25519: &str,
        their_one_time_key: &str,
    ) -> Result<()> {
        let identity = Curve25519PublicKey::from_base64(their_curve25519)
            .map_err(|e| P2pError::Crypto(e.to_string()))?;
        let otk = Curve25519PublicKey::from_base64(their_one_time_key)
            .map_err(|e| P2pError::Crypto(e.to_string()))?;

        let session = self
            .account
            .create_outbound_session(OlmSessionConfig::version_1(), identity, otk)
            .map_err(|e| P2pError::Crypto(e.to_string()))?;

        self.olm.insert(their_curve25519.to_owned(), session);
        Ok(())
    }

    pub fn olm_encrypt(&mut self, their_curve25519: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let session = self
            .olm
            .get_mut(their_curve25519)
            .ok_or_else(|| P2pError::Crypto("no Olm session for peer".into()))?;

        // Fix for E0599 & E0308: `encrypt` returns a Result, handle it.
        let msg = session
            .encrypt(plaintext)
            .map_err(|e| P2pError::Crypto(e.to_string()))?;

        // Fix for E0599: `OlmMessage` doesn't have `to_bytes()`, but its inner `Message` does.
        match msg {
            OlmMessage::Normal(m) => Ok(m.to_bytes()),
            OlmMessage::PreKey(m) => Ok(m.to_bytes()),
        }
    }

    pub fn olm_decrypt(&mut self, their_curve25519: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let msg = vodozemac::olm::PreKeyMessage::from_bytes(ciphertext)
            .map(OlmMessage::PreKey)
            .or_else(|_| {
                vodozemac::olm::Message::from_bytes(ciphertext).map(OlmMessage::Normal)
            })
            .map_err(|e| P2pError::Crypto(e.to_string()))?;

        if let OlmMessage::PreKey(ref prekey) = msg {
            let their_key = Curve25519PublicKey::from_base64(their_curve25519)
                .map_err(|e| P2pError::Crypto(e.to_string()))?;
            // Fixed: pass the owned key, not a reference
            let result = self
                .account
                .create_inbound_session(OlmSessionConfig::version_1(), their_key, prekey)
                .map_err(|e| P2pError::Crypto(e.to_string()))?;
            self.olm.insert(their_curve25519.to_owned(), result.session);
            // Fixed: plaintext is already Vec<u8>, return directly
            return Ok(result.plaintext);
        }

        let session = self
            .olm
            .get_mut(their_curve25519)
            .ok_or_else(|| P2pError::Crypto("no Olm session for peer".into()))?;
        session
            .decrypt(&msg)
            .map(|v| v.to_vec())
            .map_err(|e| P2pError::Crypto(e.to_string()))
    }

    pub fn megolm_session_key(&mut self, topic: &str) -> Result<Vec<u8>> {
        let session = self
            .outbound
            .entry(topic.to_owned())
            .or_insert_with(|| GroupSession::new(SessionConfig::version_1()));

        Ok(session.session_key().to_bytes())
    }

    pub fn megolm_encrypt(&mut self, topic: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let session = self
            .outbound
            .entry(topic.to_owned())
            .or_insert_with(|| GroupSession::new(SessionConfig::version_1()));

        let msg = session.encrypt(plaintext);
        // Fix: MegolmMessage has `to_bytes()`.
        Ok(msg.to_bytes())
    }

    pub fn add_inbound_megolm(
        &mut self,
        topic: &str,
        sender_curve25519: &str,
        session_key_bytes: &[u8],
    ) -> Result<()> {
        let key = SessionKey::from_bytes(session_key_bytes)
            .map_err(|e| P2pError::Crypto(e.to_string()))?;

        // Fix for E0061: `InboundGroupSession::new` takes 2 arguments.
        let inbound = InboundGroupSession::new(&key, SessionConfig::version_1());

        self.inbound
            .insert(format!("{topic}/{sender_curve25519}"), inbound);
        Ok(())
    }

    pub fn megolm_decrypt(
        &mut self,
        topic: &str,
        sender_curve25519: &str,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>> {
        let key = format!("{topic}/{sender_curve25519}");
        let inbound = self
            .inbound
            .get_mut(&key)
            .ok_or_else(|| P2pError::Crypto("no inbound Megolm session".into()))?;

        // Fix: Use MegolmMessage::from_bytes().
        let msg = MegolmMessage::from_bytes(ciphertext)
            .map_err(|e| P2pError::Crypto(e.to_string()))?;

        inbound
            .decrypt(&msg)
            .map(|r| r.plaintext.into())
            .map_err(|e| P2pError::Crypto(e.to_string()))
    }
}

impl Default for CryptoEngine {
    fn default() -> Self {
        Self::new()
    }
}