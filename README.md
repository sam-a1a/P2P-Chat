# P2P-Chat

Multi-platform P2P chat app using libp2p and Rust.

## Modules

| Module | Description |
|--------|-------------|
| `Rust-Core` | Shared Rust library + desktop chat node (TCP + WebRTC) |
| `Android-Kotlin` | Android chat app (connects via TCP direct dial) |
| `Web-React` | Browser chat UI (React + Tailwind CSS) |
| `Web-WASM` | Browser libp2p peer compiled to WebAssembly |

## What Works

- Desktop <-> Android chat over TCP (direct dial, fixed port 53493).  
  Messages are exchanged on topic `chat`. mDNS is enabled but unreliable on some networks, so Android dials the desktop's hardcoded IP.

- Desktop node listens on both TCP (port 53493) and WebRTC (port 9090/udp).  
  Android uses TCP only.

- Android app loads native `.so` libraries built from `Rust-Core`; JNI symbols exported correctly with `#[unsafe(no_mangle)]` and `--export-dynamic`.  
  The app works reliably between two Android devices (via mDNS) or between Android and desktop (via direct dial).

- Web-React UI is functional, Tailwind CSS works, Vite serves WASM module correctly.  
  The browser dials the desktop's WebRTC address and initiates a connection.

- Web-WASM compiles to WASM and runs in the browser. It creates a libp2p swarm with WebRTC transport, subscribes to `chat`, and polls events.

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

### Desktop node
```bash
cd Rust-Core
cargo run
The node will print its TCP and WebRTC listen addresses.

Android app
cd Android-Kotlin
./gradlew assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk

Make sure the desktop IP (192.168.1.6) is correct in Rust-Core/src/ffi/mod.rs. Rebuild the native library with bash build-android.sh if the IP changes.

Web app
cd Web-WASM
wasm-pack build --target web --out-dir ../Web-React/src/wasm

cd ../Web-React
npm install
npm run dev

Update DESKTOP_WEBRTC_ADDR in src/App.tsx to match the desktop's WebRTC address (including the certhash).

Notes
The repo .gitignore excludes node_modules, dist, and generated .so / .wasm files.

The Android jniLibs directory is ignored; native libraries must be rebuilt with build-android.sh.

Desktop IP is currently hardcoded – change it in Rust-Core/src/ffi/mod.rs (for Android) and Web-React/src/App.tsx (for Web) if your laptop's IP changes.


