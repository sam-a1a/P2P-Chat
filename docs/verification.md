# Verification status

Verified on 2026-10-16 after the dependency upgrades. Scope: foreground, shared-channel LAN chat using the implemented desktop CLI, React, Solid 2.0 RC, and Android clients.

## Automated checks

- Five Rust tests pass: identity uniqueness/persistence/Unix permissions and corrupt-key preservation; null C-ABI inputs; bidirectional TCP; bidirectional WebSocket; invalid dial and publish-without-peers errors.
- `cargo fmt --all -- --check` and warning-free Clippy pass. CI enforces both.
- React and Solid production builds and TypeScript checks pass.
- Three Chromium/Playwright scenarios pass: each browser client exchanges messages with a real desktop peer, rejects invalid/unreachable addresses, disconnects and reconnects; React and Solid exchange messages through the desktop. Readiness uses subscription events, not handshake sleeps.
- [Final CI run 38901256832](https://github.com/sam-a1a/P2P-Chat/actions/runs/38901256832) passed on commit `e8c13c4`: Linux/macOS native tests, formatting and Clippy, browser builds/E2E, Android arm64 native build, APK assembly, and Android lint. This implementation was merged into `main` as `9c251ed` in [PR #8](https://github.com/sam-a1a/P2P-Chat/pull/8).

## Android runtime evidence

The arm64 debug APK from [run 38899934817](https://github.com/sam-a1a/P2P-Chat/actions/runs/38899934817), commit `6c9ccb0` was installed and launched on both an Android 36 arm64 emulator and a physical Samsung SM-S721B phone.

- Emulator explicitly connected to the Mac desktop via `/ip4/10.0.2.2/tcp/53493`.
- The phone discovered the desktop through mDNS on their shared LAN.
- Desktop messages appeared in both Android UIs.
- Messages sent from each Android UI appeared in the desktop terminal and the other Android UI; local publication cleared the input only after acceptance.
- Force-stopping/relaunching the emulator app preserved its identity file and allowed reconnection.
- The rendered phone UI was inspected. These are foreground checks; background delivery is not implemented.

## Limits

Not verified: sustained load/soak testing, hostile networks, Windows desktop runtime, internet/NAT traversal, secure hosted WebSocket deployment, or Android ABIs other than arm64. Native Linux networking is exercised in CI. Solid 2.0 remains a release candidate. Planned Svelte/iOS/Flutter/native desktop GUI clients are not implemented.

These checks establish the current LAN baseline, not universal production readiness. Publication acceptance is not a remote-delivery acknowledgement. See [architecture](architecture.md) and the root README for transport and security boundaries.
