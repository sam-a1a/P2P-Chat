# Architecture

Native nodes run `core` with TCP/Noise/Yamux and WebSocket/Noise/Yamux listeners. React and Solid load `web/wasm`, which runs a real libp2p browser peer over browser WebSockets. Gossipsub carries signed messages on the `chat` topic; identify exchanges protocol metadata. Browsers cannot accept arbitrary TCP connections or use native mDNS, so they dial a reachable native peer. Two browsers can exchange messages through that peer.

`web/shared/client.ts` owns initialization, polling, connection timeout, shutdown, and restart cancellation. Framework components render the same events. A local message is shown only after gossipsub accepts publication. This is not a remote delivery acknowledgement. Network errors are surfaced; reconnect is explicit through Connect.

Android calls the engine through JNI. A single lock protects all native handle access, including polling and destruction. A ViewModel collects events before starting the node and retains UI state across configuration changes. Native errors travel as serialized events. C FFI exists for future integrations; the caller must serialize handle access and never use a destroyed pointer.

WebSocket is the supported browser transport. The previous README's desktop WebRTC listener was absent from source, so merely upgrading DTLS could not fix connectivity. WebRTC, relay/hole-punching, and internet-wide discovery remain future work.

## Why not gRPC?

The central requirement is peer messaging and discovery. gRPC is suitable for a known service API; it does not provide libp2p's discovery/identity/network overlay. Official gRPC-Web currently lacks bidirectional streaming. A future backend may use gRPC without replacing the peer transport; Protobuf schemas can also be used independently.

References: [libp2p SwarmBuilder](https://docs.rs/libp2p/latest/libp2p/struct.SwarmBuilder.html), [gRPC-Web streaming](https://github.com/grpc/grpc-web#streaming-support).
