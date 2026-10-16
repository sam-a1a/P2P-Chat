# P2P-Chat

Multi-platform P2P chat app using libp2p and Rust.

## Modules

| Module | Description |
|--------|-------------|
| `core` | Shared Rust library + desktop chat node (TCP + WebRTC) |
| `mobile/native-android` | Android chat app (connects via TCP direct dial) |
| `web/react` | Browser chat UI (React + Tailwind CSS) |
| `web/wasm` | Browser libp2p peer compiled to WebAssembly |

## What Works

- Desktop <-> Android chat over TCP (direct dial, fixed port 53493).  
  Messages are exchanged on topic `chat`. mDNS is enabled but unreliable on some networks, so Android dials the desktop's hardcoded IP.

- Desktop node listens on both TCP (port 53493) and WebRTC (port 9090/udp).  
  Android uses TCP only.

- Android app loads native `.so` libraries built from `core`; JNI symbols exported correctly with `#[unsafe(no_mangle)]` and `--export-dynamic`.  
  The app works reliably between two Android devices (via mDNS) or between Android and desktop (via direct dial).

- web/react UI is functional, Tailwind CSS works, Vite serves WASM module correctly.  
  The browser dials the desktop's WebRTC address and initiates a connection.

- web/wasm compiles to WASM and runs in the browser. It creates a libp2p swarm with WebRTC transport, subscribes to `chat`, and polls events.

## Not Yet Working

- Browser <-> desktop chat via WebRTC – the connection fails during the DTLS handshake.  
  Logs show:ICE connection state changed: connected
Failed to start manager dtls: invalid named curve

This is caused by a known bug in the `webrtc-rs` crate ([webrtc-rs/webrtc#417](https://github.com/webrtc-rs/webrtc/issues/417)).  
The browser sends a list of supported elliptic curves; the current Rust implementation only checks the *first* one and rejects the handshake if it's not recognised - even though the list also contains P-256 / P-384 / X25519, which *are* supported.

## Planned Fix (Next Steps)

1. Upgrade `libp2p-webrtc` to a version that includes the fix from `webrtc-rs` v0.13.0 (once it is published on crates.io).  
 *Alternatively* patch the dependency locally using a fork or a path override.

2. Fallback to WebSocket transport – if WebRTC remains unstable, add a WebSocket listener on the desktop and use `libp2p-websocket-websys` in the browser. WebSockets are more stable for browser-to-server connections and still allow P2P via the same gossipsub overlay.

3. Remove hardcoded IP addresses – add a simple UI to input the desktop's address (or use mDNS in the browser via WebRTC signalling).

## How to Run

Prerequisites: Rust stable (edition 2024), Node.js 24+, and the WebAssembly target (`rustup target add wasm32-unknown-unknown`). Android also needs JDK 17+, the Android SDK/NDK, and cargo-ndk.

### Desktop Node
```bash
cargo run -p p2p-chat-cli
```
The node prints its TCP and WebRTC listen addresses. Use the WebRTC address (including the certhash) for the browser client.

### Android App
```bash
cd mobile/native-android
./gradlew assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
```
- Ensure the desktop IP (`192.168.1.6`) is correct in `core/src/ffi/mod.rs`.  
- Rebuild the native library if the IP changes: `bash build-android.sh`.

### Web App
```bash
cd web/react
npm ci
npm run build
npm run dev
```
- Update `DESKTOP_WEBRTC_ADDR` in `src/App.tsx` to the desktop’s WebRTC address (including the certhash).

## Notes
- The `.gitignore` excludes `node_modules/`, `dist/`, and generated `.so` / `.wasm` files.
- The `mobile/native-android/app/src/main/jniLibs/` directory is ignored; native libraries must be rebuilt with `build-android.sh`.
- The desktop IP is currently hardcoded – change it in `core/src/ffi/mod.rs` (for Android) and `web/react/src/App.tsx` (for Web) when the laptop’s IP changes.

### Build checks

```bash
cargo test --locked -p p2p
cd web/react && npm ci && npm run build
```

`web/wasm` is intentionally a separate Cargo workspace with its own lockfile, so native dependencies are not compiled for wasm32. `npm run build` generates the WASM bindings before bundling the UI. `npm run build:wasm` refreshes only those bindings. CI runs desktop tests and the production browser build. A successful build does not establish browser/desktop transport compatibility; see issue #2.

### Peer identities

Each installation generates its own `identity.key`. Keep it private; losing it changes your peer ID. Keys included in older revisions are public and must not be reused. Existing users should stop the node, remove the old root/core/Rust-Core `identity.key` obtained from this repository, and restart. Git history has not been rewritten. New Unix keys have owner-only permissions.
