# Verification status

Local verification on macOS Apple Silicon, 2026-10-16, during the current implementation:

- Rust: identity uniqueness/persistence/permissions; corrupt-key handling; bidirectional TCP and WebSocket exchanges; invalid dialing and publish-without-peers errors passed.
- React: production build and real browser/desktop send/receive, invalid address, disconnect/reconnect passed.
- Solid 2.0 RC: real browser/desktop send/receive, invalid address, disconnect/reconnect passed; type checking is included in the final build check.
- React ↔ Solid through a native desktop peer passed.
- Android native build/APK/emulator checks are in progress.
- Dependency upgrades and CI verification are in progress; the above checks must be rerun after upgrades.

Not verified: physical Android devices, separate machines/networks, Windows/Linux desktop runtime, internet/NAT traversal, secure hosted WebSocket deployment. Planned Svelte/iOS/Flutter/native desktop GUI clients are not implemented. No production-readiness claim is made.
