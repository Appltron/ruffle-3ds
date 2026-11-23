# Architecture notes

## Design principle

The target is a native Horizon OS application, not a browser running on the 3DS and not remote streaming. Hardware-specific code should be isolated behind narrow interfaces so the runtime can be evaluated independently.

## Proposed layers

- **Platform/frontend:** libctru or Rust3DS/ctru-rs; lifecycle, filesystem, input polling, screens and audio device.
- **SWF loader:** file access, decompression, tag parsing and validation.
- **Runtime:** timeline, display list, events, AVM1/ActionScript 1 and 2; later assess AVM2/AS3.
- **Renderer:** translate Flash display-list operations into batched 2D drawing suitable for the PICA200.
- **Input adapter:** touchscreen-to-stage mouse coordinates and configurable key mapping.
- **Audio adapter:** Flash audio events to native 3DS audio output.

## Ruffle integration strategy

Ruffle is a Rust Flash Player emulator with desktop and WebAssembly frontends. We should not assume the desktop or WebAssembly frontend is reusable on 3DS. First map the workspace crates and dependencies, then identify whether the core runtime can compile for `armv6k-nintendo-3ds`.

Potential blockers to investigate:

- Rust target support and partially supported `std` APIs.
- Crates that assume desktop OS facilities, threads, filesystem semantics or dynamic libraries.
- Renderer abstractions and GPU backends unavailable on the 3DS.
- Audio backends that assume desktop APIs.
- Font/text shaping dependencies and asset sizes.
- Ruffle's builtin ActionScript class generation, especially the AS3 build pipeline.
- Heap fragmentation, memory ceilings and CPU cost on the O3DS ARM11.

Do not fork or rewrite large parts of Ruffle until a small proof-of-concept demonstrates which pieces can be reused.

## Rendering

The top screen is the default Flash stage. The bottom screen is the control/configuration surface, with optional stage mirroring or debug information. Stage scaling should preserve aspect ratio by default, with a user-selectable fit/crop mode.

## Input

The bottom touchscreen is the primary pointer. Convert touch coordinates to stage coordinates after applying the chosen scale/offset transform. Touch start/end should generate mouse button transitions; stylus movement while pressed should generate pointer movement. Physical buttons are configurable key events rather than hard-coded game-specific behavior.

## Performance policy

- O3DS is the baseline target; N3DS-specific improvements must be optional.
- Prefer profiling and workload-specific optimisation over assumptions that all Flash content is lightweight.
- Keep allocations predictable; avoid per-frame heap churn.
- Batch draw calls and cache immutable assets where memory permits.
- Use fixed-point arithmetic only where measurements and correctness justify it.
- Report unsupported SWF features clearly rather than silently corrupting playback.
