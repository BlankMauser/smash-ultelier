Smash Ultelier is primarily an interface for smash plugins to the GPU, but will hopefully grow into a tool for quicker reverse engineering of smash ultimate data structs.

Multiple plugins should be able to talk to the same `ssbusync`
and `imgui-smash` without crashing.

Most of the time, you should treat `SmashUltelier` as a code library, not as a standalone plugin.

## Typical usage

If another plugin only needs the shared `ssbusync` control API, depend on
`ultelier` as a library and disable the standalone plugin pieces:

```toml
[dependencies]
ultelier = { path = "../SmashUltelier", default-features = false, features = ["sync-guest"] }
```

Then use the re-exported guest API:

```rust
use ultelier::sync_guest::{self as sync, BufferMode, IndexBackend};

pub fn enable_dynamic_triple_buffer() {
    let _ = sync::set_index_backend(IndexBackend::Dynamic);
    let _ = sync::set_buffer_mode(BufferMode::Triple);
}
```

If you are working directly with the guest crate instead of the root re-export:

```toml
[dependencies]
ssbusync_guest = { package = "ssbusync-guest", path = "../SmashUltelier/crates/sync-guest" }
```

## Runtime double/triple switching

This is the main reason to use the library.

Use `set_buffer_mode(...)` for real runtime transitions:

```rust
use ultelier::sync_guest::{self as sync, BufferMode, IndexBackend};

#[derive(Default)]
struct BufferController {
    last_mode: Option<BufferMode>,
}

impl BufferController {
    fn apply(&mut self, desired: BufferMode) {
        let _ = sync::set_index_backend(IndexBackend::Dynamic);

        if self.last_mode == Some(desired) {
            return;
        }

        if sync::set_buffer_mode(desired) == Some(true) {
            self.last_mode = Some(desired);
        }
    }
}
```

Basic toggle example:

```rust
use ultelier::sync_guest::{self as sync, BufferMode};

pub fn set_low_latency_mode(enabled: bool) {
    let target = if enabled {
        BufferMode::Double
    } else {
        BufferMode::Triple
    };

    let _ = sync::set_buffer_mode(target);
}
```

You can also use the convenience helper:

```rust
let _ = ultelier::sync_guest::set_triple_buffer_enabled(true);
let _ = ultelier::sync_guest::set_triple_buffer_enabled(false);
```

## Important startup requirement

If you want to switch between triple and double buffer at runtime, `ssbusync`
must start with triple buffering enabled.

Reason:

- triple buffer needs the third window texture to be allocated up front
- switching from triple -> double at runtime is fine
- switching from double -> triple later is not safe if the third texture was
  never allocated during startup

In practice:

- start `ssbusync` in triple-buffer mode
- use the dynamic backend
- then switch down to double buffer at runtime when you want lower latency

Recommended startup sequence from a guest plugin:

```rust
use ultelier::sync_guest::{self as sync, BufferMode, IndexBackend};

pub fn initialize_sync_control() {
    let _ = sync::set_index_backend(IndexBackend::Dynamic);
    let _ = sync::set_buffer_mode(BufferMode::Triple);
}
```

That keeps the runtime in the correct mode for later triple/double transitions.

## Notes

- `set_frame_index_mode(...)` is intentionally internal/debug-only; normal code
  should use `set_buffer_mode(...)`
- if `ssbusync` is not loaded, guest calls return `None`
- if you need a local debug UI, that lives behind the `plugin` feature, but that
  is not the primary purpose of this repo
