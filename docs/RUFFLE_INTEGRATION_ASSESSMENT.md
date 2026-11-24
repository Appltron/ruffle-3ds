# Ruffle integration assessment

## Scope and source snapshot

Initial source review of the upstream `ruffle-rs/ruffle` repository at commit
`2c6d3c4fe97d5c6f8c447edc2ddafa3b4abb6c66` (the source snapshot reviewed for this assessment).

This is an initial architecture map, not a complete audit of every transitive dependency.

## Confirmed crate boundaries

- `swf`: standalone SWF file-format crate. It exposes `decompress_swf` and `parse_swf`; it does not require the player, renderer, or ActionScript virtual machines to parse a movie's structure.
- `ruffle_core`: the player/runtime. It contains both AVM1 (ActionScript 1/2) and AVM2 (ActionScript 3) modules.
- `ruffle_render`: rendering data structures and the `RenderBackend` interface.
- `ruffle_video` and `flv-rs`: video and Flash Video support.
- Desktop and web frontends are separate workspace members; their windowing, browser and GPU integrations are not the native 3DS frontend we need.

## Useful architectural discovery

Ruffle's `PlayerBuilder` accepts platform backends, including renderer, audio, navigator, storage, UI, video and locale. This is encouraging: the 3DS frontend can potentially provide native implementations rather than embedding a desktop or browser frontend.

However, `RenderBackend` is a substantial interface, not a small framebuffer callback. It covers shape and bitmap registration, frame submission, offscreen rendering, filters, textures, Context3D and Pixel Bender. A practical 3DS renderer will need a deliberate first supported subset and explicit handling of unsupported features.

## Confirmed build considerations

- `ruffle_core` directly depends on `ruffle_render` with its `tessellator` feature, plus `ruffle_video`, `swf`, `flv-rs`, and many general-purpose crates.
- Core's `audio` feature is optional and is not enabled by its default feature set. This gives us a potential way to defer audio decoding while bringing up the runtime, though the core still exposes audio backend interfaces.
- Core's build script generates both AVM1 and AVM2 player globals. The AVM2 generation calls the `asc` helper, which launches an external `java` process to run `asc.jar`. Therefore Java is a build-time requirement for the current core build path, even though our initial game-compatibility priority is AS1/AS2.
- The core source includes both AVM1 and AVM2; selecting AS1/AS2 as the first compatibility target does not currently mean Cargo can omit the AVM2 implementation.
- The upstream SWF parser supports compressed SWFs through optional `flate2` and `lzma-rs` features. For the first target experiment, disable default features and test an uncompressed FWS fixture; compression can be added after the basic crate builds.

## Main unknowns / risks

1. Whether the complete `swf` crate and its dependency graph compile for `armv6k-nintendo-3ds`.
2. Whether `ruffle_core` and its full dependency graph compile for this target, including platform-sensitive APIs, entropy, clocks, filesystem and networking assumptions.
3. Whether the O3DS memory and CPU budgets can support a useful subset of Ruffle.
4. How much of the `RenderBackend` contract can be implemented efficiently with the 3DS PICA200 GPU / native graphics stack.
5. Which audio, font, video, networking and storage features should be stubbed or deferred for the first playable AS1/AS2 tier.

A successful Rust smoke build does not resolve any of these runtime or performance questions.

## Recommended integration order

1. Cross-compile the upstream `swf` crate for the 3DS target, with a tiny in-memory FWS fixture. This isolates parsing from the player and renderer.
2. If successful, add a small SD-card read path and report SWF metadata on the 3DS screen.
3. Attempt a separate `ruffle_core` compile-only experiment with minimal features and the required Java build dependency. Do not integrate a renderer yet.
4. Only after core compilation succeeds, prototype a native renderer with a deliberately small supported feature set.
5. Add input, audio and broader AS1/AS2 compatibility incrementally.

## Current decision

The next engineering test is the SWF parser cross-compile. It is a smaller and more diagnostic step than attempting the entire Ruffle core immediately. The existing C and minimal Rust smoke builds remain separate baselines and should not be destabilized.
