use super::callback::Callback;
use super::{BufferMode, IndexBackend, StateCallback};
use std::sync::Mutex;

pub type TypedVsyncCallback = extern "C" fn(bool);
pub type TypedBufferModeCallback = extern "C" fn(BufferMode);
pub type TypedIndexBackendCallback = extern "C" fn(IndexBackend);

static TYPED_VSYNC_CHANGED: Mutex<Callback<TypedVsyncCallback>> = Mutex::new(Callback::new(None));
static TYPED_BUFFER_MODE_CHANGED: Mutex<Callback<TypedBufferModeCallback>> =
    Mutex::new(Callback::new(None));
static TYPED_INDEX_BACKEND_CHANGED: Mutex<Callback<TypedIndexBackendCallback>> =
    Mutex::new(Callback::new(None));

fn with_typed_callback<F, R>(
    slot: &Mutex<Callback<F>>,
    f: impl FnOnce(&mut Callback<F>) -> R,
) -> R {
    let mut callback = slot.lock().unwrap_or_else(|err| err.into_inner());
    f(&mut callback)
}

extern "C" fn vsync_changed_typed_thunk(enabled: u32) {
    let enabled = enabled != 0;
    with_typed_callback(&TYPED_VSYNC_CHANGED, |callback| {
        let _ = callback.invoke((enabled,));
    });
}

extern "C" fn buffer_mode_changed_typed_thunk(raw: u32) {
    let Some(mode) = BufferMode::from_u32(raw) else {
        return;
    };
    with_typed_callback(&TYPED_BUFFER_MODE_CHANGED, |callback| {
        let _ = callback.invoke((mode,));
    });
}

extern "C" fn index_backend_changed_typed_thunk(raw: u32) {
    let Some(mode) = IndexBackend::from_u32(raw) else {
        return;
    };
    with_typed_callback(&TYPED_INDEX_BACKEND_CHANGED, |callback| {
        let _ = callback.invoke((mode,));
    });
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffectEvent {
    VsyncChanged = 0,
    BufferModeChanged = 1,
    IndexBackendChanged = 2,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SideEffectRegistry {
    pub vsync_changed: Callback<StateCallback>,
    pub buffer_mode_changed: Callback<StateCallback>,
    pub index_backend_changed: Callback<StateCallback>,
}

impl SideEffectRegistry {
    /// Registers every populated raw callback in the remote runtime.
    ///
    /// # Example
    /// ```ignore
    /// use ultelier::sync_guest::callback::Callback;
    /// use ultelier::sync_guest::events::SideEffectRegistry;
    ///
    /// extern "C" fn on_vsync(_: u32) {}
    ///
    /// let registry = SideEffectRegistry {
    ///     vsync_changed: Callback::from_fn(on_vsync),
    ///     ..Default::default()
    /// };
    ///
    /// let ok = registry.register_remote();
    /// ```
    pub fn register_remote(&self) -> bool {
        let mut ok = true;
        if self.vsync_changed.is_set() {
            ok &= super::set_vsync_changed_callback(self.vsync_changed.get()) == Some(true);
        }
        if self.buffer_mode_changed.is_set() {
            ok &= super::set_buffer_mode_changed_callback(self.buffer_mode_changed.get())
                == Some(true);
        }
        if self.index_backend_changed.is_set() {
            ok &= super::set_index_backend_changed_callback(self.index_backend_changed.get())
                == Some(true);
        }
        ok
    }

    /// Clears all raw callbacks from the remote runtime.
    ///
    /// # Example
    /// ```ignore
    /// let ok = ultelier::sync_guest::events::SideEffectRegistry::clear_remote();
    /// ```
    pub fn clear_remote() -> bool {
        super::clear_vsync_changed_callback() == Some(true)
            && super::clear_buffer_mode_changed_callback() == Some(true)
            && super::clear_index_backend_changed_callback() == Some(true)
    }
}

/// Registers a raw vsync-change callback and returns whether registration
/// succeeded.
///
/// # Example
/// ```ignore
/// extern "C" fn on_vsync_changed(raw: u32) {
///     skyline::println!("vsync enabled = {}", raw != 0);
/// }
///
/// let ok = ultelier::sync_guest::events::set_vsync_changed(on_vsync_changed);
/// ```
pub fn set_vsync_changed(callback: StateCallback) -> bool {
    super::set_vsync_changed_callback(Some(callback)) == Some(true)
}

/// Clears the raw vsync-change callback.
///
/// # Example
/// ```ignore
/// let ok = ultelier::sync_guest::events::clear_vsync_changed();
/// ```
pub fn clear_vsync_changed() -> bool {
    super::clear_vsync_changed_callback() == Some(true)
}

/// Registers a raw buffer-mode callback and returns whether registration
/// succeeded.
///
/// # Example
/// ```ignore
/// extern "C" fn on_buffer_mode_changed(raw: u32) {
///     skyline::println!("buffer mode raw = {raw}");
/// }
///
/// let ok = ultelier::sync_guest::events::set_buffer_mode_changed(on_buffer_mode_changed);
/// ```
pub fn set_buffer_mode_changed(callback: StateCallback) -> bool {
    super::set_buffer_mode_changed_callback(Some(callback)) == Some(true)
}

/// Clears the raw buffer-mode callback.
///
/// # Example
/// ```ignore
/// let ok = ultelier::sync_guest::events::clear_buffer_mode_changed();
/// ```
pub fn clear_buffer_mode_changed() -> bool {
    super::clear_buffer_mode_changed_callback() == Some(true)
}

/// Registers a raw index-backend callback and returns whether registration
/// succeeded.
///
/// # Example
/// ```ignore
/// extern "C" fn on_index_backend_changed(raw: u32) {
///     skyline::println!("index backend raw = {raw}");
/// }
///
/// let ok = ultelier::sync_guest::events::set_index_backend_changed(on_index_backend_changed);
/// ```
pub fn set_index_backend_changed(callback: StateCallback) -> bool {
    super::set_index_backend_changed_callback(Some(callback)) == Some(true)
}

/// Clears the raw index-backend callback.
///
/// # Example
/// ```ignore
/// let ok = ultelier::sync_guest::events::clear_index_backend_changed();
/// ```
pub fn clear_index_backend_changed() -> bool {
    super::clear_index_backend_changed_callback() == Some(true)
}

/// Registers a typed `bool` callback for vsync changes.
///
/// # Example
/// ```ignore
/// extern "C" fn on_vsync_changed(enabled: bool) {
///     skyline::println!("vsync enabled = {enabled}");
/// }
///
/// let ok = ultelier::sync_guest::events::set_typed_vsync_changed(on_vsync_changed);
/// ```
pub fn set_typed_vsync_changed(callback: TypedVsyncCallback) -> bool {
    with_typed_callback(&TYPED_VSYNC_CHANGED, |slot| {
        let _ = slot.set(callback);
    });
    super::set_vsync_changed_callback(Some(vsync_changed_typed_thunk)) == Some(true)
}

/// Clears the typed vsync callback.
///
/// # Example
/// ```ignore
/// let ok = ultelier::sync_guest::events::clear_typed_vsync_changed();
/// ```
pub fn clear_typed_vsync_changed() -> bool {
    with_typed_callback(&TYPED_VSYNC_CHANGED, |slot| {
        let _ = slot.clear();
    });
    super::clear_vsync_changed_callback() == Some(true)
}

/// Registers a typed `BufferMode` callback for buffer-mode changes.
///
/// # Example
/// ```ignore
/// use ultelier::sync_guest::BufferMode;
///
/// extern "C" fn on_buffer_mode_changed(mode: BufferMode) {
///     skyline::println!("buffer mode = {:?}", mode);
/// }
///
/// let ok = ultelier::sync_guest::events::set_typed_buffer_mode_changed(on_buffer_mode_changed);
/// ```
pub fn set_typed_buffer_mode_changed(callback: TypedBufferModeCallback) -> bool {
    with_typed_callback(&TYPED_BUFFER_MODE_CHANGED, |slot| {
        let _ = slot.set(callback);
    });
    super::set_buffer_mode_changed_callback(Some(buffer_mode_changed_typed_thunk)) == Some(true)
}

/// Clears the typed buffer-mode callback.
///
/// # Example
/// ```ignore
/// let ok = ultelier::sync_guest::events::clear_typed_buffer_mode_changed();
/// ```
pub fn clear_typed_buffer_mode_changed() -> bool {
    with_typed_callback(&TYPED_BUFFER_MODE_CHANGED, |slot| {
        let _ = slot.clear();
    });
    super::clear_buffer_mode_changed_callback() == Some(true)
}

/// Registers a typed `IndexBackend` callback for index-backend changes.
///
/// # Example
/// ```ignore
/// use ultelier::sync_guest::IndexBackend;
///
/// extern "C" fn on_index_backend_changed(mode: IndexBackend) {
///     skyline::println!("index backend = {:?}", mode);
/// }
///
/// let ok = ultelier::sync_guest::events::set_typed_index_backend_changed(on_index_backend_changed);
/// ```
pub fn set_typed_index_backend_changed(callback: TypedIndexBackendCallback) -> bool {
    with_typed_callback(&TYPED_INDEX_BACKEND_CHANGED, |slot| {
        let _ = slot.set(callback);
    });
    super::set_index_backend_changed_callback(Some(index_backend_changed_typed_thunk)) == Some(true)
}

/// Clears the typed index-backend callback.
///
/// # Example
/// ```ignore
/// let ok = ultelier::sync_guest::events::clear_typed_index_backend_changed();
/// ```
pub fn clear_typed_index_backend_changed() -> bool {
    with_typed_callback(&TYPED_INDEX_BACKEND_CHANGED, |slot| {
        let _ = slot.clear();
    });
    super::clear_index_backend_changed_callback() == Some(true)
}
