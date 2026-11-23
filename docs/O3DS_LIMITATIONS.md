# Original 3DS constraints and validation

The original 3DS (O3DS) is the compatibility baseline. Exact usable memory and CPU budget depend on system state, application memory layout and whether extended-memory modes are available; do not hard-code optimistic desktop-like assumptions.

## Working assumptions to validate

- ARM11-class CPU and PICA200 GPU.
- Native homebrew runs under Horizon OS through libctru.
- The Rust target `armv6k-nintendo-3ds` exists, but Rust support is Tier 3 and some standard-library APIs are unavailable or incomplete.
- A successful cross-compile does not prove runtime correctness or adequate performance.

## Required measurements

For each test SWF, record:

- SWF version, compression, stage dimensions and frame rate.
- AS version/features used.
- Peak heap usage and allocation failures.
- Average and worst frame time.
- Number of display objects, vector paths, bitmap dimensions and audio streams.
- Input latency and audio underruns.

## Test tiers

1. Parser-only fixtures: no renderer or ActionScript execution.
2. Static vector/bitmap content.
3. Simple timeline animation.
4. Basic mouse-driven AS1/AS2 games.
5. More demanding AS2 games with audio and many display objects.
6. AS3 experiments, only after earlier tiers are stable.

## Hardware validation

Emulator tests are useful for fast iteration but cannot establish real O3DS performance. Final performance claims require measurements on the original hardware. Until the SD-card/homebrew setup is repaired, mark device-level behavior as **untested**.
