# Ruffle3DS

A native Nintendo 3DS Flash-player research project. The initial target is the **original 3DS (O3DS)**; New 3DS is a secondary performance target, not a requirement.

> **Status: bootstrapping.** This repository currently aims to prove a reproducible native 3DS build. It does not yet contain a Ruffle port or play SWF files.

## Project goals

- Run locally on 3DS hardware; no streaming or remote execution.
- Investigate reusing Ruffle's Rust ActionScript/SWF runtime.
- Prioritize ActionScript 1/2 compatibility and O3DS performance.
- Treat ActionScript 3 as a later, separately benchmarked goal.
- Use the bottom touchscreen as a stylus-driven mouse, with configurable button-to-key mappings.
- Build reproducibly in GitHub Actions; keep the developer's laptop free of the console toolchain.

## Current milestone

The first milestone is a tiny libctru application that builds into a `.3dsx` artifact in GitHub Actions. It is a toolchain smoke test, not evidence that Ruffle or Rust has been ported.

## Build

Push a commit or manually run **Actions → Build 3DS homebrew**. Download the `ruffle3ds-homebrew` artifact from the completed workflow. The artifact contains the `.3dsx` and `.elf` outputs.

The build uses the official devkitPro devkitARM container and libctru. No local devkitPro installation is required.

## Planned architecture

1. Native libctru frontend and dual-screen application shell.
2. Rust3DS/cargo-3ds feasibility spike for the target `armv6k-nintendo-3ds`.
3. SWF parsing and timeline playback proof-of-concept.
4. Ruffle core integration feasibility, beginning with the smallest dependency surface.
5. Native renderer, audio and input backends as needed.
6. AS1/AS2 compatibility and O3DS profiling; AS3 investigation after the baseline.

See [ROADMAP.md](docs/ROADMAP.md), [ARCHITECTURE.md](docs/ARCHITECTURE.md), and [O3DS_LIMITATIONS.md](docs/O3DS_LIMITATIONS.md).

## Important technical caveat

Ruffle is written in Rust, but its current supported frontends are desktop and WebAssembly. A 3DS port is substantial systems work: target support alone does not guarantee that Ruffle's dependencies, renderer, audio stack, threading assumptions, or memory use will work on Horizon OS. We will validate each layer rather than claim that the existing project can simply be cross-compiled.

## Licensing

Ruffle is licensed under the MIT License. This repository's original code and documentation will be licensed separately once the project structure and any vendored/linked dependencies are settled. Do not copy Ruffle source into this repository without preserving its license and notices.
