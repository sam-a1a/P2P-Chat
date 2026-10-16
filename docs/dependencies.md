# Dependency maintenance

Versions were checked against npm, crates.io, Google Maven, Maven Central, Gradle release metadata, and GitHub releases on 2026-10-16. Lockfiles are committed for reproducibility; Dependabot opens weekly ecosystem-specific updates.

- libp2p: 0.57.1.
- React/React DOM: 19.3.1.
- Solid/web runtime: 2.0.0-rc.15 (explicitly requested prerelease); Solid Vite plugin 3.0.0-next.50.
- TypeScript: 7.0.2; Vite: 8.3.4; React Vite plugin: 6.1.2.
- Kotlin: 2.4.21; AGP: 9.4.1; Gradle: 9.8.1.
- AndroidX Core: 1.19.1; Lifecycle: 2.11.0; Activity: 1.13.0; Compose BOM: 2026.09.00.

Compatibility exception: libp2p-swarm 0.48.0 explicitly pins wasm-bindgen-futures 0.4.58. The browser consequently uses wasm-bindgen 0.2.108 and js-sys/web-sys 0.3.85 rather than their newer independently published releases. Forcing newer bindings fails Cargo dependency resolution. Update this family together when upstream lifts the pin.

Transitive versions are selected by upstream constraints; “updated” does not mean every transitive package can be independently forced to its highest published major. All upgrade changes require the native tests, browser builds/E2E checks, and affected Android build checks before release.
