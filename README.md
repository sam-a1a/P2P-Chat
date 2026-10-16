# P2P Chat

A multi-platform LAN chat project built on Rust and libp2p. Desktop and Android peers use TCP; browsers connect to a reachable native peer over WebSockets. React and Solid share the same Rust/WASM transport and TypeScript connection lifecycle.

## Repository map

| Path | Purpose | State |
| --- | --- | --- |
| `core/` | Rust networking, identities, JNI/C FFI | Implemented |
| `desktop/cli/` | Desktop command-line peer | Implemented |
| `web/wasm/` | Browser libp2p peer | Implemented |
| `web/shared/` | Browser lifecycle, events, styling | Shared by both UIs |
| `web/react/` | React chat client | Implemented |
| `web/solid/` | Solid **2.0 RC** chat client | Implemented; prerelease framework |
| `mobile/native-android/` | Kotlin/Compose + JNI | Implemented |
| `web/svelte/` | Svelte client | Planned |
| `mobile/native-ios/`, `mobile/flutter/` | iOS and Flutter clients | Planned |
| `desktop/native-{macos,windows,linux}/` | Native desktop GUIs | Planned |

The planned folders contain documentation only. They are not runnable applications.

## Run desktop + browser

Prerequisites: Rust stable with edition 2024 support, Node.js 24+, npm, and `rustup target add wasm32-unknown-unknown`.

```bash
# Terminal 1, repository root
cargo run -p p2p-chat-cli --locked

# Terminal 2
cd web/react
npm ci
npm run build
npm run dev
```

Open **http://127.0.0.1:5173**. Keep the default `/ip4/127.0.0.1/tcp/9090/ws` address and select **Connect**. Send from the browser or type into the desktop terminal. To run Solid, use the same commands in `web/solid`; its URL is **http://127.0.0.1:5174**.

The build generates `web/wasm/pkg` before type-checking and bundling the UI. `npm run build:wasm` refreshes the shared WASM bindings; `npm run preview` serves the production build. The first build downloads Rust and WASM tooling. npm configurations that block install scripts must explicitly allow the `wasm-pack` installer.

For another machine on your LAN, substitute the desktop's LAN IP in the peer address. The desktop listens on TCP **53493** and WebSocket TCP **9090** by default. Allow those ports through your firewall. An HTTPS-hosted UI needs a secure `wss` endpoint; the local plain-WebSocket setup is intended for development/LAN use. See [architecture](docs/architecture.md).

### Desktop options

```bash
cargo run -p p2p-chat-cli -- --help
cargo run -p p2p-chat-cli -- \
  --identity /tmp/second-peer/identity.key \
  --tcp /ip4/127.0.0.1/tcp/53494 \
  --websocket /ip4/127.0.0.1/tcp/9091/ws \
  --dial /ip4/127.0.0.1/tcp/53493
```

Use a distinct identity path and ports for each local node. Type `quit` to stop. mDNS discovers LAN peers; explicit dialing works when multicast discovery is unavailable.

## Android

Requires JDK 21 or newer (set `JAVA_HOME`), Android SDK 37.0, Android NDK 29.0.13846066, and Rust Android targets.

```bash
cargo install cargo-ndk --locked
export ANDROID_NDK_HOME=/path/to/android/sdk/ndk/29.0.13846066
bash core/build-android.sh
cd mobile/native-android
./gradlew assembleDebug lintDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n com.example.p2p/.MainActivity
```

The native build script accepts optional Rust targets, e.g. `bash core/build-android.sh aarch64-linux-android`. Full builds produce arm64, armv7, and x86_64 libraries. APK builds require those native libraries to provide messaging; Gradle does not generate them automatically.

In the app, enter `/ip4/DESKTOP_LAN_IP/tcp/53493` and tap **Connect**. The Android emulator can use `/ip4/10.0.2.2/tcp/53493`. Keep the app open during chat; background delivery is not implemented.

## Checks

```bash
cargo test --workspace --locked
cargo build -p p2p-chat-cli --locked
npm ci
npm --prefix web/react ci
npm --prefix web/solid ci
npm run build:web
npx playwright install chromium
npm run test:e2e
```

Native tests exercise TCP/WebSocket exchanges, failures, and identity persistence. Browser tests launch an isolated real desktop peer and verify React/Solid messaging, invalid addresses, disconnect, reconnect, and cross-client exchange. See [verification](docs/verification.md) for the current evidence and remaining limits.

## Security and scope

This is an online **shared-channel LAN chat**, not a production private messenger. Transport uses Noise encryption and messages have libp2p signatures. All peers subscribed to `chat` can read and forward messages. There is no room membership control, user authentication, offline delivery, message history UI, NAT traversal setup, or integrated end-to-end group encryption. `core/src/crypto` and `storage` are separate utilities, not part of the current message path. The field named `ciphertext` in native events currently carries plain message bytes after transport decryption.

Each native installation generates an `identity.key`. Keep it private; losing it changes your peer ID. Keys committed in old revisions are public and must not be reused. Stop the node, remove old root/core/Rust-Core keys obtained from the repository, and restart. New Unix keys are owner-only; Android excludes its key from backups. History has not been rewritten.

See [contributing](CONTRIBUTING.md) and [open issues](https://github.com/sam-a1a/P2P-Chat/issues). Work is tracked in focused issues and commits; a passing build is not a claim that every planned platform is implemented.
