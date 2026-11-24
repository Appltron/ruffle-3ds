# Roadmap

## Phase 0 — Reproducible native build

- [x] Create project repository and initial documentation.
- [x] Build a minimal libctru application in GitHub Actions.
- [x] Confirm the workflow produces a non-empty `.3dsx` and ELF.
- [x] Inspect the artifact and record compiler/toolchain versions.

**Exit condition:** CI build succeeds and publishes the homebrew artifact. This validates the C/libctru toolchain only.

## Phase 1 — Rust3DS feasibility

- [ ] Establish a separate GitHub Actions Rust build without changing the working C/libctru workflow.
- [ ] Install nightly Rust with `rust-src` and build using `cargo-3ds`.
- [ ] Build a minimal `ctru-rs` application for `armv6k-nintendo-3ds`.
- [ ] Verify both the Rust ELF and packaged `.3dsx` output.
- [ ] Pin the known-good Rust nightly and cargo-3ds versions.
- [ ] Record limitations in standard-library support, allocator, panic handling, threading and linker requirements.

**Exit condition:** a minimal Rust application builds reproducibly in CI and produces a non-empty `.3dsx` and ELF. Do not start porting Ruffle before this works.

**Confirmed platform facts:** Rust lists this target as Tier 3. It has partial `std` support, requires cross-compilation, and does not support dynamic linking. The Rust project recommends `cargo-3ds` and `ctru-rs` for building and supplying 3DS runtime/libctru integration.

## Phase 2 — SWF foundation

- [ ] Select a small set of legally distributable SWF test fixtures.
- [ ] Implement or integrate SWF header/tag parsing.
- [ ] Display metadata (version, dimensions, frame rate, compression) in the native frontend.
- [ ] Establish memory and timing instrumentation on O3DS-compatible builds.

## Phase 3 — Runtime and rendering research

- [ ] Map Ruffle workspace crates and target-specific dependencies.
- [ ] Identify the smallest reusable ActionScript/SWF runtime surface.
- [ ] Prototype a minimal timeline/shape renderer.
- [ ] Compare a native 2D renderer against a possible Citro2D/Citro3D backend.
- [ ] Measure CPU, memory and frame time before optimising.

## Phase 4 — Interaction

- [ ] Stylus position maps to Flash stage coordinates.
- [ ] Touch-down/up maps to mouse button events.
- [ ] Configurable 3DS buttons map to common Flash keyboard keys.
- [ ] Add pause, exit, scaling and stage positioning controls.

## Phase 5 — Compatibility

- [ ] AS1/AS2 compatibility test matrix.
- [ ] Prioritise common 2D games and their actual API usage.
- [ ] Profile and optimise on original 3DS hardware when available.
- [ ] Investigate AS3 separately; its builtin classes and runtime requirements may be a substantially larger challenge.

## Phase 6 — Release engineering

- [ ] SD-card file browser and SWF launching.
- [ ] Configuration persistence.
- [ ] Audio support.
- [ ] Compatibility reporting and reproducible release artifacts.
