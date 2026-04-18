Smash Ultelier is primarily an interface for smash plugins to the game's render loop but will hopefully grow into a tool for quicker reverse engineering of smash ultimate data structs.

Ideally multiple plugins should be able to talk to the same `ssbusync` without crashing but its
recommended to only have one plugin interface with the buffer settings.

Most users should treat `SmashUltelier` as a code library, not as a standalone plugin.

## Typical usage

Import `ultelier` as a library and disable the standalone plugin feature:

```toml
[dependencies]
ultelier = { path = "", default-features = false, features = ["sync-guest"] }
```

Then use the re-exported guest API:

```rust
use ultelier::sync_guest::{self as sync, BufferMode, IndexBackend};

pub fn enable_dynamic_triple_buffer() {
    let _ = sync::set_index_backend(IndexBackend::Dynamic);
    let _ = sync::set_buffer_mode(BufferMode::Triple);
}
```

## Runtime double/triple switching

This is the main reason to use the library.

Use `set_buffer_mode(...)` to switch between 3 delay and 4 delay (better performance).
Currently going back to default delay is buggy and not supported. It is unlikely I make
a fix for that personally.

Recommended startup from a guest plugin:

```rust
use ultelier::sync_guest::{self as sync, BufferMode, IndexBackend};

pub fn initialize_sync_control() {
    let _ = sync::set_index_backend(IndexBackend::Dynamic);
    let _ = sync::set_buffer_mode(BufferMode::Triple);
}
```

IndexBackend has to be Dynamic and triple buffer must start on. Otherwise you will not
have the memory allocated to switch between triple/double buffers.

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

That keeps the runtime in the correct mode for later triple/double transitions.

## Callbacks

`sync_guest::events` is the subscription API for runtime change notifications.

- `set_*` / `set_typed_*` returns `bool`: `true` if the callback was registered with the remote runtime, `false` means registration failed.
- The callback itself does not return a success value. It is just invoked when the state changes.

Event subscription example:

```rust
use ultelier::sync_guest::{self as sync, events, BufferMode, IndexBackend};

extern "C" fn on_buffer_mode_changed(mode: BufferMode) {
    skyline::println!("buffer mode changed to {:?}", mode);
}

extern "C" fn on_index_backend_changed(mode: IndexBackend) {
    skyline::println!("index backend changed to {:?}", mode);
}

pub fn subscribe_to_sync_callbacks() -> bool {
    events::set_typed_buffer_mode_changed(on_buffer_mode_changed)
        && events::set_typed_index_backend_changed(on_index_backend_changed)
}
```

To unsubscribe:

```rust
let _ = ultelier::sync_guest::events::clear_typed_buffer_mode_changed();
let _ = ultelier::sync_guest::events::clear_typed_index_backend_changed();
let _ = ultelier::sync_guest::events::clear_typed_vsync_changed();
```
