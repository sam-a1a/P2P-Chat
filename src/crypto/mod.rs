//! End-to-end encryption layer.
//!
//! Design
//! Olm (vodozemac) authenticated 1-to-1 key exchange.
//! Used to securely distribute Megolm session keys to individual peers.
//!
//! Megolm (vodozemac) — group ratchet encryption.
//! One outbound GroupSession per Gossipsub topic each peer shares their
//! SessionKey with the group via an Olm-encrypted key-exchange message
//! on the reserved topic __keyex
//!
//! The transport is already Noise-encrypted this layer adds E2E
//! forward-secrecy on top so the node itself cannot read message content.

use crate::error::{P2pError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vodozemac::{
    megolm::{
        DecryptionError, GroupSession, GroupSessionConfig, InboundGroupSession,
        MegolmMessage, SessionConfig as MegolmSessionConfig, SessionKey,
    },
    olm::{Account, InboundCreationResult, OlmMessage, Session, SessionConfig as OlmSessionConfig},
    Curve25519PublicKey,
};
use zeroize::Zeroizing;

// Public key bundle

/// Published on __keyex so peers can initiate Olm sessions with us.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyBundle {
    /// Base64-encoded Curve25519 identity key.
    pub curve25519: String,
    /// Base64-encoded one-time prekeys (consumed one per new session).
    pub one_time_keys: Vec<String>,
}

// Engine

pub struct CryptoEngine {
    account: Account,
    /// Olm sessions keyed by the remote's Curve25519 identity key (base64).
    olm: HashMap<String, Session>,
    /// Outbound Megolm sessions, one per topic name.
    outbound: HashMap<String, GroupSession>,
    /// Inbound Megolm sessions, keyed by "<topic>/<sender_curve25519>".
    inbound: HashMap<String, InboundGroupSession>,
}

impl CryptoEngine {
    pub fn new() -> Self {
        let mut account = Account::new();
        // Pre-generate a pool of one-time keys.
        account.generate_one_time_keys(20);
        Self {
            account,
            olm: HashMap::new(),
            outbound: HashMap::new(),
            inbound: HashMap::new(),
        }
    }

    // Identity

    /// Returns the bundle to advertise to newly connected peers.
    /// Marks the current one-time keys as published after reading them.
    pub fn public_bundle(&mut self) -> PublicKeyBundle {
        let curve25519 = self.account.curve25519_key().to_base64();
        let one_time_keys: Vec<String> = self
            .account
            .one_time_keys()
            .values()
            .map(|k| k.to_base64())
            .collect();
        self.account.mark_keys_as_published();

        // Top up the pool so we're never empty.
        if one_time_keys.len() < 5 {
            self.account.generate_one_time_keys(20);
        }

        PublicKeyBundle { curve25519, one_time_keys }
    }

    // Olm (key exchange)

    /// Creates an outbound Olm session to a peer using their identity key
    /// and one-time key.  Call this before `olm_encrypt`.
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
            .create_outbound_session(OlmSessionConfig::version_2(), &identity, &otk);

        self.olm.insert(their_curve25519.to_owned(), session);
        Ok(())
    }

    /// Encrypts plaintext via an existing Olm session.
    /// Returns postcard-serialised OlmMessage bytes.
    pub fn olm_encrypt(&mut self, their_curve25519: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let session = self
            .olm
            .get_mut(their_curve25519)
            .ok_or_else(|| P2pError::Crypto("no Olm session for peer".into()))?;

        let msg = session.encrypt(plaintext);
        postcard::to_stdvec(&msg).map_err(|e| P2pError::Serialization(e.to_string()))
    }

    /// Decrypts an inbound Olm message.
    /// If this is the first PreKey message, an inbound session is created
    /// automatically.  Returns plaintext bytes.
    pub fn olm_decrypt(&mut self, their_curve25519: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let msg: OlmMessage = postcard::from_bytes(ciphertext)
            .map_err(|e| P2pError::Serialization(e.to_string()))?;

        if let OlmMessage::PreKey(ref prekey) = msg {
            let their_key = Curve25519PublicKey::from_base64(their_curve25519)
                .map_err(|e| P2pError::Crypto(e.to_string()))?;
            let InboundCreationResult { session, plaintext } = self
                .account
                .create_inbound_session(&their_key, prekey)
                .map_err(|e| P2pError::Crypto(e.to_string()))?;
            self.olm.insert(their_curve25519.to_owned(), session);
            return Ok(plaintext.into());
        }

        // Normal message — use the existing session.
        let session = self
            .olm
            .get_mut(their_curve25519)
            .ok_or_else(|| P2pError::Crypto("no Olm session for peer".into()))?;
        session
            .decrypt(&msg)
            .map(|v| v.to_vec())
            .map_err(|e| P2pError::Crypto(e.to_string()))
    }

    // Megolm (group / Gossipsub)

    /// Returns the serialised SessionKey for topic.
    /// Share this (Olm-encrypted) with every peer that joins the topic.
    /// Creates a new GroupSession for the topic if one does not exist.
    pub fn megolm_session_key(&mut self, topic: &str) -> Result<Vec<u8>> {
        let session = self
            .outbound
            .entry(topic.to_owned())
            .or_insert_with(|| GroupSession::new(GroupSessionConfig::version_1()));

        postcard::to_stdvec(&session.session_key())
            .map_err(|e| P2pError::Serialization(e.to_string()))
    }

    /// Encrypts plaintext for broadcasting on topic
    pub fn megolm_encrypt(&mut self, topic: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let session = self
            .outbound
            .entry(topic.to_owned())
            .or_insert_with(|| GroupSession::new(GroupSessionConfig::version_1()));

        let msg = session.encrypt(plaintext);
        postcard::to_stdvec(&msg).map_err(|e| P2pError::Serialization(e.to_string()))
    }

    /// Registers an inbound Megolm session from a peer's shared SessionKey
    /// session_key_bytes is the postcard-serialised bytes from megolm_session_key
    pub fn add_inbound_megolm(
        &mut self,
        topic: &str,
        sender_curve25519: &str,
        session_key_bytes: &[u8],
    ) -> Result<()> {
        let key: SessionKey = postcard::from_bytes(session_key_bytes)
            .map_err(|e| P2pError::Serialization(e.to_string()))?;

        let inbound = InboundGroupSession::new(&key, MegolmSessionConfig::version_1())
            .map_err(|e| P2pError::Crypto(e.to_string()))?;

        self.inbound
            .insert(format!("{topic}/{sender_curve25519}"), inbound);
        Ok(())
    }

    /// Decrypts a Gossipsub message from sender_curve25519 on topic
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

        let msg: MegolmMessage = postcard::from_bytes(ciphertext)
            .map_err(|e| P2pError::Serialization(e.to_string()))?;

        inbound
            .decrypt(&msg)
            .map(|r| r.plaintext.into())
            .map_err(|e| P2pError::Crypto(e.to_string()))
    }
}

impl Default for CryptoEngine {
    fn default() -> Self { Self::new() }
}