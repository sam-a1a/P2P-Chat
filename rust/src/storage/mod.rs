use crate::{
    error::{P2pError, Result},
    types::ChatMessage,
};
use rusqlite::{params, Connection};

pub struct MessageStore {
    conn: Connection,
}

impl MessageStore {
    pub fn open(path: &str) -> Result<Self> {
        let conn =
            Connection::open(path).map_err(|e| P2pError::Storage(e.to_string()))?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS messages (
                 id             TEXT PRIMARY KEY,
                 from_peer      TEXT    NOT NULL,
                 topic          TEXT    NOT NULL,
                 ciphertext     BLOB    NOT NULL,
                 timestamp_secs INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_messages_topic
                 ON messages(topic, timestamp_secs);",
        )
            .map_err(|e| P2pError::Storage(e.to_string()))?;

        Ok(Self { conn })
    }

    /// Persists a message. Silently ignores duplicate IDs (INSERT OR IGNORE).
    pub fn insert(&self, msg: &ChatMessage) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO messages
                     (id, from_peer, topic, ciphertext, timestamp_secs)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    msg.id,
                    msg.from_peer,
                    msg.topic,
                    msg.ciphertext,
                    msg.timestamp_secs as i64,
                ],
            )
            .map_err(|e| P2pError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Returns all messages for topic, oldest first.
    pub fn for_topic(&self, topic: &str) -> Result<Vec<ChatMessage>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, from_peer, topic, ciphertext, timestamp_secs
                 FROM messages
                 WHERE topic = ?1
                 ORDER BY timestamp_secs ASC",
            )
            .map_err(|e| P2pError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![topic], |row| {
                Ok(ChatMessage {
                    id:             row.get(0)?,
                    from_peer:      row.get(1)?,
                    topic:          row.get(2)?,
                    ciphertext:     row.get(3)?,
                    timestamp_secs: row.get::<_, i64>(4)? as u64,
                })
            })
            .map_err(|e| P2pError::Storage(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| P2pError::Storage(e.to_string()))?;

        Ok(rows)
    }

    /// Deletes messages older than older_than_secs (Unix timestamp) for a
    /// topic.  Call periodically from mobile to cap storage usage.
    pub fn prune(&self, topic: &str, older_than_secs: u64) -> Result<usize> {
        let n = self
            .conn
            .execute(
                "DELETE FROM messages
                 WHERE topic = ?1 AND timestamp_secs < ?2",
                params![topic, older_than_secs as i64],
            )
            .map_err(|e| P2pError::Storage(e.to_string()))?;
        Ok(n)
    }
}