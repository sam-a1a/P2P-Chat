# Contributing

Open a focused issue with the observed behavior, expected behavior, reproduction steps, and acceptance checks. Keep platform moves, dependency upgrades, protocol changes, and UI changes in separate commits. Include relevant issue numbers in commit/PR descriptions.

Run the checks in the root README for the affected clients. Rust changes must also pass `cargo fmt --all -- --check` and `cargo clippy --workspace --locked --all-targets -- -D warnings`. For networking changes, verify real peer exchanges; a successful compile is insufficient. For Android changes, build native libraries before the APK and report the emulator/device used. Never mark planned clients as implemented or claim production readiness from build-only evidence.

Commit both Cargo lockfiles and each npm lockfile. Do not commit private identities, databases, native libraries, WASM output, node_modules, or SDK paths. Keep the platform/status table and run commands synchronized with code changes.
