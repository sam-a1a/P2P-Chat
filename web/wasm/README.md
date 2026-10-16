# Browser peer

Shared libp2p WebSocket/Noise/Yamux implementation used by React and Solid. Build through either client's `npm run build:wasm`; generated bindings are written to `pkg/` and ignored by Git.

This crate is intentionally excluded from the native Cargo workspace and has its own committed lockfile. Its local Cargo configuration enables browser randomness. Run Cargo from this directory so that configuration is loaded.

Exports: `start_node(address)`, `send_message(id, text)`, `poll_event()`, and `shutdown()`. Startup validates the address before publishing node state. Events distinguish connected/disconnected peers, received messages, accepted publications, and failures. The shared TypeScript client owns polling, cancellation, and timeout handling.

libp2p 0.57.1 pins wasm-bindgen-futures 0.4.58 and the corresponding bindings family. Keep those versions compatible; upgrading individual binding crates beyond the upstream pin makes resolution fail.
