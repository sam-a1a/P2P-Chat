# P2P Chat

[![CI](https://github.com/sam-a1a/P2P-Chat/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/sam-a1a/P2P-Chat/actions/workflows/ci.yml)

Peer-to-peer text chat for devices on the same local network, powered by Rust and libp2p. Run a desktop peer, connect from a browser or Android phone, and exchange messages on a shared channel. Native peers can discover each other automatically with mDNS; explicit addresses work when discovery is unavailable.

## What works today

- Desktop CLI ↔ Android over TCP, with Noise transport encryption and signed messages.
- Browser ↔ native peer over WebSockets, including browser-to-browser messages through that peer.
- React and Solid **2.0 RC** clients with editable peer addresses, connection status, error reporting, and disconnect/reconnect controls.
- Android foreground chat with configurable dialing, retained UI state, and private per-installation identity.
- Local send confirmation after publication is accepted. This is not confirmation that another device received the message.

**Verified LAN baseline:** Linux/macOS native checks, both browser production builds, Chromium messaging/reconnect tests, and Android arm64 APK assembly/lint pass. Two-way messaging was also verified between a Mac, an Android emulator, and a physical Samsung phone on the same LAN. The implementation is merged into `main`; [verification details](docs/verification.md) record the tested revisions and remaining limits.

## Quick start

Install Rust stable with edition 2024 support and Node.js 24+ with npm. Then clone and prepare the browser target:

```bash
git clone https://github.com/sam-a1a/P2P-Chat.git
cd P2P-Chat
rustup target add wasm32-unknown-unknown
```

In one terminal, start a desktop peer:

```bash
cargo run -p p2p-chat-cli --locked
```

In a second terminal, from the repository root, choose a browser client:

| Client | Install and build | Start | URL |
| --- | --- | --- | --- |
| React | `npm --prefix web/react ci && npm --prefix web/react run build` | `npm --prefix web/react run dev` | http://127.0.0.1:5173 |
| Solid 2.0 RC | `npm --prefix web/solid ci && npm --prefix web/solid run build` | `npm --prefix web/solid run dev` | http://127.0.0.1:5174 |

Open the chosen URL, keep `/ip4/127.0.0.1/tcp/9090/ws`, and select **Connect**. Wait for **connected**, then send a message. Type a reply in the desktop terminal. Type `quit` there to stop the desktop peer.

The build generates the shared WASM bindings before type checking and bundling. Run the client's `build:wasm` script after changing the Rust browser transport. The first build downloads WASM tooling; npm setups that block install scripts must allow the `wasm-pack` installer.

## Connect devices on your LAN

Keep the desktop peer running and substitute its LAN IP in the address:

| Connecting client | Peer address |
| --- | --- |
| Browser on the desktop machine | `/ip4/127.0.0.1/tcp/9090/ws` |
| Browser on another LAN machine | `/ip4/DESKTOP_LAN_IP/tcp/9090/ws` |
| Physical Android phone or another desktop peer | `/ip4/DESKTOP_LAN_IP/tcp/53493` |
| Android emulator on the desktop machine | `/ip4/10.0.2.2/tcp/53493` |

For a browser on another computer, run its web client locally using the quick-start commands and change the **Peer address** field. Browser peers dial a native peer; they do not use native mDNS or accept TCP connections.

Allow TCP **53493** and **9090** through the desktop firewall. Both listeners bind to all interfaces by default. An HTTPS-hosted UI requires a secure `wss` endpoint; the supplied local setup uses plain WebSockets with Noise-encrypted peer traffic. See [architecture](docs/architecture.md).

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

## Desktop options

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
cargo install cargo-ndk --version 4.1.2 --locked
export ANDROID_NDK_HOME=/path/to/android/sdk/ndk/29.0.13846066
bash core/build-android.sh
cd mobile/native-android
./gradlew assembleDebug lintDebug
adb install --no-streaming -r app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n com.example.p2p/.MainActivity
```

The native build script accepts optional Rust targets, e.g. `bash core/build-android.sh aarch64-linux-android`. Full builds produce arm64, armv7, and x86_64 libraries. APK builds require those native libraries to provide messaging; Gradle does not generate them automatically.

Successful [CI runs](https://github.com/sam-a1a/P2P-Chat/actions/workflows/ci.yml) also provide an `android-debug-arm64` APK artifact. It is a development build for arm64 devices, not a signed store release. With multiple devices attached, select one using `adb -s DEVICE_SERIAL`.

In the app, enter `/ip4/DESKTOP_LAN_IP/tcp/53493` and tap **Connect**. The Android emulator can use `/ip4/10.0.2.2/tcp/53493`. Keep the app open during chat; background delivery is not implemented.

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --locked --all-targets -- -D warnings
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

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Connection times out | The native peer is running, the address uses its reachable LAN IP, and the firewall allows the correct port. |
| Browser cannot connect | Use port `9090` with `/ws`; the native TCP port `53493` is for Android/desktop peers. |
| Android cannot connect with its default address | `10.0.2.2` is an emulator address. On a phone, enter the desktop's LAN IP or use automatic LAN discovery. |
| Send is disabled | Wait for the peer's chat subscription to be ready. A socket connection alone is insufficient. |
| Android reports a missing native library | Run `core/build-android.sh` for the device ABI before assembling the APK. |
| Port is already in use | Stop the other node or supply different CLI listener ports and a distinct identity path. |

## Dependencies and maintenance

Dependencies and toolchains were updated on 2026-10-16. Lockfiles pin reproducible builds, and Dependabot checks supported ecosystems weekly. Solid uses `2.0.0-rc.15`; upstream libp2p constraints require compatible WASM binding versions rather than independently selecting the newest binding packages. See [dependency versions and exceptions](docs/dependencies.md).

## Security and scope

This is an online **shared-channel LAN chat**, not a production private messenger. Transport uses Noise encryption and messages have libp2p signatures. All peers subscribed to `chat` can read and forward messages. There is no room membership control, user authentication, offline delivery, message history UI, NAT traversal setup, or integrated end-to-end group encryption. `core/src/crypto` and `storage` are separate utilities, not part of the current message path. The field named `ciphertext` in native events currently carries plain message bytes after transport decryption.

Each native installation generates an `identity.key`. Keep it private; losing it changes your peer ID. Keys committed in old revisions are public and must not be reused. Stop the node, remove old root/core/Rust-Core keys obtained from the repository, and restart. New Unix keys are owner-only; Android excludes its key from backups. History has not been rewritten.

See [contributing](CONTRIBUTING.md), [architecture](docs/architecture.md), and [issues](https://github.com/sam-a1a/P2P-Chat/issues). Keep changes scoped to focused issues and commits. The completed LAN baseline was merged in [PR #8](https://github.com/sam-a1a/P2P-Chat/pull/8); the planned platform clients above remain future work.
