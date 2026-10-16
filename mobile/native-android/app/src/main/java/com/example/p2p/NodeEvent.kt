package com.example.p2p

import org.json.JSONObject

sealed class NodeEvent {
    data class PeerDiscovered(val peer: String) : NodeEvent()
    data class PeerExpired(val peer: String) : NodeEvent()
    data class ConnectionEstablished(val peer: String, val address: String) : NodeEvent()
    data class ConnectionClosed(val peer: String) : NodeEvent()
    data class MessageReceived(
        val id: String,
        val fromPeer: String,
        val topic: String,
        val ciphertext: ByteArray,
        val timestampSecs: Long
    ) : NodeEvent()
    data class ListeningOn(val address: String) : NodeEvent()
    data class Error(val message: String) : NodeEvent()
    object Unknown : NodeEvent()

    companion object {
        fun fromJson(json: String): NodeEvent {
            val obj = JSONObject(json)
            return when {
                obj.has("PeerDiscovered") ->
                    PeerDiscovered(obj.getJSONObject("PeerDiscovered").getString("peer"))

                obj.has("PeerExpired") ->
                    PeerExpired(obj.getJSONObject("PeerExpired").getString("peer"))

                obj.has("ConnectionEstablished") -> {
                    val inner = obj.getJSONObject("ConnectionEstablished")
                    ConnectionEstablished(inner.getString("peer"), inner.getString("address"))
                }

                obj.has("ConnectionClosed") ->
                    ConnectionClosed(obj.getJSONObject("ConnectionClosed").getString("peer"))

                obj.has("MessageReceived") -> {
                    val inner = obj.getJSONObject("MessageReceived")
                    MessageReceived(
                        id = inner.getString("id"),
                        fromPeer = inner.getString("from_peer"),
                        topic = inner.getString("topic"),
                        ciphertext = android.util.Base64.decode(
                            inner.getString("ciphertext"), android.util.Base64.DEFAULT
                        ),
                        timestampSecs = inner.getLong("timestamp_secs")
                    )
                }

                obj.has("ListeningOn") ->
                    ListeningOn(obj.getJSONObject("ListeningOn").getString("address"))

                obj.has("Error") ->
                    Error(obj.getJSONObject("Error").getString("message"))

                else -> Unknown
            }
        }
    }
}