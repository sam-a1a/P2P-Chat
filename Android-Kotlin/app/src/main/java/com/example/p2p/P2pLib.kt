package com.example.p2p

object P2pLib {

    init {
        System.loadLibrary("p2p")
    }

    /**
     * Start the P2P node.
     * @param keyPath path to identity key file
     * @param dbPath path to SQLite database file
     * @return opaque handle (pointer) to the node, 0 on failure
     */
    external fun start(keyPath: String, dbPath: String): Long

    /**
     * Poll for the next event from the node.
     * @param handle node handle returned by [start]
     * @return JSON string of the event, or null if no event
     */
    external fun pollEvent(handle: Long): String?

    /**
     * Subscribe to a gossipsub topic.
     * @param handle node handle
     * @param topic topic name
     */
    external fun subscribe(handle: Long, topic: String)

    /**
     * Publish data to a gossipsub topic.
     * @param handle node handle
     * @param topic topic name
     * @param data raw bytes to publish
     */
    external fun publish(handle: Long, topic: String, data: ByteArray)

    /**
     * Shutdown the P2P node gracefully.
     * @param handle node handle
     */
    external fun shutdown(handle: Long)

    /**
     * Destroy and free the node handle.
     * Must be called after [shutdown].
     * @param handle node handle
     */
    external fun destroy(handle: Long)
}