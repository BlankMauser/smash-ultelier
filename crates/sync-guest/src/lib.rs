use skyline::nn::ro;

pub mod callback;
pub mod events;
pub mod profile;
pub mod runtime;

pub const SSBUSYNC_STATUS_SYMBOL: &[u8] = b"ssbusync_status\0";
pub const SSBUSYNC_ENV_GET_FLAGS_SYMBOL: &[u8] = b"ssbusync_env_get_flags\0";
pub const SSBUSYNC_ENV_REPLACE_FLAGS_SYMBOL: &[u8] = b"ssbusync_env_replace_flags\0";
pub const SSBUSYNC_ENV_SET_FLAG_SYMBOL: &[u8] = b"ssbusync_env_set_flag\0";
pub const SSBUSYNC_SET_VSYNC_ENABLED_SYMBOL: &[u8] = b"ssbusync_set_vsync_enabled\0";
pub const SSBUSYNC_SET_PACER_ENABLED_SYMBOL: &[u8] = b"ssbusync_set_pacer_enabled\0";
pub const SSBUSYNC_SET_TRIPLE_BUFFER_ENABLED_SYMBOL: &[u8] =
    b"ssbusync_set_triple_buffer_enabled\0";
pub const SSBUSYNC_SET_BUFFER_MODE_SYMBOL: &[u8] = b"ssbusync_set_buffer_mode\0";
pub const SSBUSYNC_SET_FRAME_INDEX_MODE_SYMBOL: &[u8] = b"ssbusync_set_frame_index_mode\0";
pub const SSBUSYNC_SET_INDEX_BACKEND_SYMBOL: &[u8] = b"ssbusync_set_index_backend\0";
pub const SSBUSYNC_SET_OVERCLOCK_PROFILE_SYMBOL: &[u8] = b"ssbusync_set_overclock_profile\0";
pub const SSBUSYNC_CURRENT_OVERCLOCK_PROFILE_SYMBOL: &[u8] =
    b"ssbusync_current_overclock_profile\0";
pub const SSBUSYNC_OVERCLOCK_USES_SAFE_PROFILES_SYMBOL: &[u8] =
    b"ssbusync_overclock_uses_safe_profiles\0";
pub const SSBUSYNC_GET_NSTUFF_STATUS_SYMBOL: &[u8] = b"ssbusync_get_nstuff_status\0";
pub const SSBUSYNC_REFRESH_INDEX_SYMBOL: &[u8] = b"ssbusync_refresh_index\0";
pub const SSBUSYNC_CURRENT_INDEX_MODE_SYMBOL: &[u8] = b"ssbusync_current_index_mode\0";
pub const SSBUSYNC_CURRENT_INDEX_BACKEND_SYMBOL: &[u8] = b"ssbusync_current_index_backend\0";
pub const SSBUSYNC_SET_VSYNC_CHANGED_CALLBACK_SYMBOL: &[u8] =
    b"ssbusync_set_vsync_changed_callback\0";
pub const SSBUSYNC_SET_BUFFER_MODE_CHANGED_CALLBACK_SYMBOL: &[u8] =
    b"ssbusync_set_buffer_mode_changed_callback\0";
pub const SSBUSYNC_SET_INDEX_BACKEND_CHANGED_CALLBACK_SYMBOL: &[u8] =
    b"ssbusync_set_index_backend_changed_callback\0";

/// Raw callback signature used by the remote ssbusync event registration API.
pub type StateCallback = extern "C" fn(u32);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnvironmentFlags(u32);

impl EnvironmentFlags {
    pub const ALLOW_BUFFER_SWAP: u32 = 1 << 0;
    pub const EMULATOR_KNOWN: u32 = 1 << 1;
    pub const EMULATOR_VALUE: u32 = 1 << 2;
    pub const SWAPPING_BUFFER: u32 = 1 << 3;
    pub const TRIPLE_ENABLED: u32 = 1 << 4;
    pub const VSYNC_DISABLED: u32 = 1 << 7;
    pub const PACER_DISABLED: u32 = 1 << 8;
    pub const PROFILING_ENABLED: u32 = 1 << 9;
    pub const SLOW_PACER_BIAS: u32 = 1 << 10;
    pub const OVERCLOCKER: u32 = 1 << 11;

    #[inline]
    pub const fn new(bits: u32) -> Self {
        Self(bits)
    }

    /// Returns the raw flag bits.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::EnvironmentFlags;
    ///
    /// let flags = EnvironmentFlags::new(EnvironmentFlags::TRIPLE_ENABLED);
    /// assert_eq!(flags.bits(), EnvironmentFlags::TRIPLE_ENABLED);
    /// ```
    #[inline]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Tests whether any bits from `mask` are set.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::EnvironmentFlags;
    ///
    /// let flags = EnvironmentFlags::new(EnvironmentFlags::TRIPLE_ENABLED);
    /// assert!(flags.contains(EnvironmentFlags::TRIPLE_ENABLED));
    /// assert!(!flags.contains(EnvironmentFlags::VSYNC_DISABLED));
    /// ```
    #[inline]
    pub const fn contains(self, mask: u32) -> bool {
        (self.0 & mask) != 0
    }

    /// Returns a copy of the flags with `mask` enabled or disabled.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::EnvironmentFlags;
    ///
    /// let flags = EnvironmentFlags::default()
    ///     .with(EnvironmentFlags::TRIPLE_ENABLED, true)
    ///     .with(EnvironmentFlags::VSYNC_DISABLED, false);
    ///
    /// assert!(flags.contains(EnvironmentFlags::TRIPLE_ENABLED));
    /// ```
    #[inline]
    pub const fn with(self, mask: u32, value: bool) -> Self {
        if value {
            Self(self.0 | mask)
        } else {
            Self(self.0 & !mask)
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Remote buffer-ring mode.
pub enum BufferMode {
    Double = 2,
    Triple = 3,
}

impl BufferMode {
    /// Converts the raw remote value into a typed `BufferMode`.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::BufferMode;
    ///
    /// assert_eq!(BufferMode::from_u32(2), Some(BufferMode::Double));
    /// assert_eq!(BufferMode::from_u32(3), Some(BufferMode::Triple));
    /// assert_eq!(BufferMode::from_u32(0), None);
    /// ```
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            2 => Some(Self::Double),
            3 => Some(Self::Triple),
            _ => None,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Remote frame-index policy.
pub enum FrameIndexMode {
    Immediate = 0,
    Double = 1,
    Triple = 2,
    Vanilla = 3,
}

impl FrameIndexMode {
    /// Return Frame Buffer Index from int
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::FrameIndexMode;
    ///
    /// assert_eq!(FrameIndexMode::from_u32(1), Some(FrameIndexMode::Double));
    /// assert_eq!(FrameIndexMode::from_u32(4), Some(FrameIndexMode::Frozen));
    /// assert_eq!(FrameIndexMode::from_u32(99), None);
    /// ```
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Immediate),
            1 => Some(Self::Double),
            2 => Some(Self::Triple),
            3 => Some(Self::Vanilla),
            _ => Some(Self::Triple),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Backend used to choose the active frame index.
pub enum IndexBackend {
    Dynamic = 0,
    Static = 1,
}

impl IndexBackend {
    /// Return Index Backend from int
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::IndexBackend;
    ///
    /// assert_eq!(IndexBackend::from_u32(0), Some(IndexBackend::Dynamic));
    /// assert_eq!(IndexBackend::from_u32(1), Some(IndexBackend::Static));
    /// assert_eq!(IndexBackend::from_u32(7), None);
    /// ```
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Dynamic),
            1 => Some(Self::Static),
            _ => None,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverclockProfile {
    PerformanceSingles = 1,
    PerformanceFfa = 2,
    Rest = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NsTuffStatus {
    pub api_version: u32,
    pub enabled: u8,
    pub has_runtime_plan: u8,
    pub require_managed_title: u8,
    pub title_match: u8,
    pub undervolt_enabled: u8,
    pub undervolt_active: u8,
    pub undervolt_cpu_mode: u8,
    pub _reserved0: u8,
    pub profile: u32,
    pub poll_interval_ms: u32,
    pub applied_cpu_hz: u32,
    pub applied_gpu_hz: u32,
    pub applied_mem_hz: u32,
    pub undervolt_cpu_vmin_mv: u32,
    pub undervolt_cpu_limit_mv: u32,
    pub undervolt_gpu_mv: u32,
}

impl OverclockProfile {
    /// Return `OverclockProfile` from int.
    ///
    /// `0` is treated as `PerformanceSingles` 
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::OverclockProfile;
    ///
    /// assert_eq!(
    ///     OverclockProfile::from_u32(2),
    ///     Some(OverclockProfile::PerformanceFfa)
    /// );
    /// assert_eq!(OverclockProfile::from_u32(5), None);
    /// ```
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 | 1 => Some(Self::PerformanceSingles),
            2 => Some(Self::PerformanceFfa),
            3 => Some(Self::Rest),
            _ => None,
        }
    }
}

fn lookup_symbol_addr(symbol: &'static [u8]) -> Option<usize> {
    let mut addr = 0usize;
    unsafe {
        if ro::LookupSymbol(&mut addr, symbol.as_ptr()) == 0 && addr != 0 {
            Some(addr)
        } else {
            None
        }
    }
}

fn call_u32(symbol: &'static [u8]) -> Option<u32> {
    let addr = lookup_symbol_addr(symbol)?;
    let func: extern "C" fn() -> u32 = unsafe { core::mem::transmute(addr) };
    Some(func())
}

fn call_u32_u32(symbol: &'static [u8], value: u32) -> Option<u32> {
    let addr = lookup_symbol_addr(symbol)?;
    let func: extern "C" fn(u32) -> u32 = unsafe { core::mem::transmute(addr) };
    Some(func(value))
}

fn call_u32_u32_u32(symbol: &'static [u8], first: u32, second: u32) -> Option<u32> {
    let addr = lookup_symbol_addr(symbol)?;
    let func: extern "C" fn(u32, u32) -> u32 = unsafe { core::mem::transmute(addr) };
    Some(func(first, second))
}

fn call_void(symbol: &'static [u8]) -> Option<()> {
    let addr = lookup_symbol_addr(symbol)?;
    let func: extern "C" fn() = unsafe { core::mem::transmute(addr) };
    func();
    Some(())
}

fn call_callback_reg(symbol: &'static [u8], callback: Option<StateCallback>) -> Option<u32> {
    let addr = lookup_symbol_addr(symbol)?;
    let func: extern "C" fn(Option<StateCallback>) -> u32 = unsafe { core::mem::transmute(addr) };
    Some(func(callback))
}

fn call_fill_struct<T: Default>(symbol: &'static [u8]) -> Option<T> {
    let addr = lookup_symbol_addr(symbol)?;
    let func: extern "C" fn(*mut T) -> u32 = unsafe { core::mem::transmute(addr) };
    let mut value = T::default();
    if func(&mut value as *mut T) != 0 {
        Some(value)
    } else {
        None
    }
}

/// Returns `true` when the remote ssbusync symbol table is available.
///
/// Warning, this can be inconsistent depending on plugin load order.
///
/// # Example
/// ```ignore
/// if !ssbusync_guest::remote_present() {
///     skyline::println!("ssbusync is not loaded");
/// }
/// ```
pub fn remote_present() -> bool {
    lookup_symbol_addr(SSBUSYNC_STATUS_SYMBOL).is_some()
}

/// Reads the remote ssbusync status.
///
/// # Example
/// ```ignore
/// if let Some(status) = ssbusync_guest::status() {
///     skyline::println!("ssbusync status = {status:#x}");
/// }
/// ```
pub fn status() -> Option<u32> {
    call_u32(SSBUSYNC_STATUS_SYMBOL)
}

/// Fetches the current environment flags for ssbusync.
///
/// # Example
/// ```ignore
/// use ssbusync_guest::EnvironmentFlags;
///
/// if let Some(flags) = ssbusync_guest::env_flags() {
///     if flags.contains(EnvironmentFlags::TRIPLE_ENABLED) {
///         skyline::println!("triple buffering is active");
///     }
/// }
/// ```
pub fn env_flags() -> Option<EnvironmentFlags> {
    call_u32(SSBUSYNC_ENV_GET_FLAGS_SYMBOL).map(EnvironmentFlags::new)
}

/// Replaces the full environment flags and returns it.
///
/// Prefer `set_env_flag` when you only need to toggle a single bit.
///
/// # Example
/// ```ignore
/// use ssbusync_guest::EnvironmentFlags;
///
/// let desired = EnvironmentFlags::default()
///     .with(EnvironmentFlags::TRIPLE_ENABLED, true)
///     .with(EnvironmentFlags::VSYNC_DISABLED, false);
///
/// let previous = ssbusync_guest::replace_env_flags(desired);
/// ```
pub fn replace_env_flags(flags: EnvironmentFlags) -> Option<EnvironmentFlags> {
    call_u32_u32(SSBUSYNC_ENV_REPLACE_FLAGS_SYMBOL, flags.bits()).map(EnvironmentFlags::new)
}

/// Toggles a single environment flag bit and returns the updated flag set.
///
/// # Example
/// ```ignore
/// use ssbusync_guest::EnvironmentFlags;
///
/// let updated = ssbusync_guest::set_env_flag(EnvironmentFlags::TRIPLE_ENABLED, true);
/// ```
pub fn set_env_flag(mask: u32, enabled: bool) -> Option<EnvironmentFlags> {
    call_u32_u32_u32(SSBUSYNC_ENV_SET_FLAG_SYMBOL, mask, u32::from(enabled))
        .map(EnvironmentFlags::new)
}

/// Enables or disables vsync in the remote runtime.
///
/// # Example
/// ```ignore
/// let applied = ssbusync_guest::set_vsync_enabled(false);
/// ```
pub fn set_vsync_enabled(enabled: bool) -> Option<bool> {
    call_u32_u32(SSBUSYNC_SET_VSYNC_ENABLED_SYMBOL, u32::from(enabled)).map(|value| value != 0)
}

/// Enables or disables the pacer in the remote runtime.
///
/// # Example
/// ```ignore
/// let applied = ssbusync_guest::set_pacer_enabled(true);
/// ```
pub fn set_pacer_enabled(enabled: bool) -> Option<bool> {
    call_u32_u32(SSBUSYNC_SET_PACER_ENABLED_SYMBOL, u32::from(enabled)).map(|value| value != 0)
}

/// Convenience wrapper for switching between the live double- and triple-buffer
/// modes exposed by ssbusync.
///
/// This is equivalent to calling `set_buffer_mode(BufferMode::Triple)` when
/// `enabled` is `true` and `set_buffer_mode(BufferMode::Double)` when `enabled`
/// is `false`.
///
/// If an external event publisher emits the desired mode every frame, do not
/// forward that event here every frame. Cache the last applied state in the
/// caller and only issue the remote call when the mode changes.
///
/// # Example
/// ```ignore
/// let applied = ssbusync_guest::set_triple_buffer_enabled(true);
/// ```
pub fn set_triple_buffer_enabled(enabled: bool) -> Option<bool> {
    call_u32_u32(
        SSBUSYNC_SET_TRIPLE_BUFFER_ENABLED_SYMBOL,
        u32::from(enabled),
    )
    .map(|value| value != 0)
}

/// Requests a live buffer-ring transition to double or triple buffering.
///
/// Use this when you want ssbusync to switch the active NVN window texture
/// count between the normal triple-buffered path and the lower-latency
/// double-buffered path.
///
/// This is different from the internal frame-index override:
/// - `set_buffer_mode(...)` changes the intended buffer-ring mode.
/// - the frame-index override changes only the frame-index policy.
///
/// For event-driven control, the fast path is edge-triggered: publish your
/// desired mode every frame if you want, but only call this function when the
/// desired value changes.
///
/// # Example
/// ```ignore
/// use ssbusync_guest::{self as sync, BufferMode};
///
/// #[derive(Default)]
/// struct BufferModeController {
///     last_applied: Option<BufferMode>,
/// }
///
/// impl BufferModeController {
///     fn apply(&mut self, desired: BufferMode) {
///         if self.last_applied == Some(desired) {
///             return;
///         }
///
///         if sync::set_buffer_mode(desired) == Some(true) {
///             self.last_applied = Some(desired);
///         }
///     }
/// }
///
/// let mut controller = BufferModeController::default();
/// controller.apply(BufferMode::Double);
/// controller.apply(BufferMode::Triple);
/// ```
pub fn set_buffer_mode(mode: BufferMode) -> Option<bool> {
    call_u32_u32(SSBUSYNC_SET_BUFFER_MODE_SYMBOL, mode as u32).map(|value| value != 0)
}

/// Overrides the frame-index policy used by ssbusync.
///
/// This does not by itself request a double/triple buffer-ring transition.
/// Use it when you explicitly want to force the frame-index mode after the
/// runtime is already active.
///
/// In most remote-control flows, call `set_buffer_mode(...)` for real
/// double/triple swaps.
///
/// This is intended for internal/debug control paths only.
/// As with `set_buffer_mode(...)`, cache the last applied value locally and
/// only forward changes across the symbol boundary.
#[doc(hidden)]
pub fn set_frame_index_mode(mode: FrameIndexMode) -> Option<bool> {
    call_u32_u32(SSBUSYNC_SET_FRAME_INDEX_MODE_SYMBOL, mode as u32).map(|value| value != 0)
}

/// Selects whether ssbusync uses its dynamic runtime or static.
///
/// You can not switch between double or triple buffer without dynamic runtime
/// enabled.
///
/// # Example
/// ```ignore
/// use ssbusync_guest::IndexBackend;
///
/// let applied = ssbusync_guest::set_index_backend(IndexBackend::Dynamic);
/// ```
pub fn set_index_backend(mode: IndexBackend) -> Option<bool> {
    call_u32_u32(SSBUSYNC_SET_INDEX_BACKEND_SYMBOL, mode as u32).map(|value| value != 0)
}

/// Requests a specific remote overclock profile.
///
/// # Example
/// ```ignore
/// use ssbusync_guest::OverclockProfile;
///
/// let applied = ssbusync_guest::set_overclock_profile(OverclockProfile::PerformanceSingles);
/// ```
pub fn set_overclock_profile(profile: OverclockProfile) -> Option<bool> {
    call_u32_u32(SSBUSYNC_SET_OVERCLOCK_PROFILE_SYMBOL, profile as u32).map(|value| value != 0)
}

/// Reads the currently selected overclock profile from the remote runtime.
///
/// The outer `Option` is `None` when the remote symbol is unavailable. The
/// inner `Option` is `None` when the remote reported an unknown profile value.
///
/// # Example
/// ```ignore
/// match ssbusync_guest::current_overclock_profile() {
///     Some(Some(profile)) => skyline::println!("active profile: {:?}", profile),
///     Some(None) => skyline::println!("active profile is unknown"),
///     None => skyline::println!("ssbusync is unavailable"),
/// }
/// ```
pub fn current_overclock_profile() -> Option<Option<OverclockProfile>> {
    call_u32(SSBUSYNC_CURRENT_OVERCLOCK_PROFILE_SYMBOL)
        .map(|value| OverclockProfile::from_u32(value))
}

/// Returns whether the remote runtime is currently constrained to safe
/// overclock profiles.
///
/// # Example
/// ```ignore
/// if ssbusync_guest::overclock_uses_safe_profiles() == Some(true) {
///     skyline::println!("safe overclock profiles are enforced");
/// }
/// ```
pub fn overclock_uses_safe_profiles() -> Option<bool> {
    call_u32(SSBUSYNC_OVERCLOCK_USES_SAFE_PROFILES_SYMBOL).map(|value| value != 0)
}

/// Fetches the current NsTuff status block from the remote runtime.
///
/// # Example
/// ```ignore
/// if let Some(status) = ssbusync_guest::current_nstuff_status() {
///     skyline::println!(
///         "nstuff cpu={} gpu={} mem={}",
///         status.applied_cpu_hz,
///         status.applied_gpu_hz,
///         status.applied_mem_hz
///     );
/// }
/// ```
pub fn current_nstuff_status() -> Option<NsTuffStatus> {
    call_fill_struct(SSBUSYNC_GET_NSTUFF_STATUS_SYMBOL)
}

/// Recomputes the runtime frame-index mode from the current environment flags.
///
/// Most callers do not need this after `set_buffer_mode(...)` or
/// `set_index_backend(...)`, because ssbusync already refreshes its runtime
/// index selection internally when those settings change.
///
/// # Example
/// ```ignore
/// if !ssbusync_guest::refresh_index() {
///     skyline::println!("could not refresh frame index state");
/// }
/// ```
pub fn refresh_index() -> bool {
    call_void(SSBUSYNC_REFRESH_INDEX_SYMBOL).is_some()
}

/// Reads the currently active frame-index mode.
///
/// # Example
/// ```ignore
/// if let Some(mode) = ssbusync_guest::current_index_mode() {
///     skyline::println!("index mode: {:?}", mode);
/// }
/// ```
pub fn current_index_mode() -> Option<FrameIndexMode> {
    call_u32(SSBUSYNC_CURRENT_INDEX_MODE_SYMBOL).and_then(FrameIndexMode::from_u32)
}

/// Reads the currently active index backend.
///
/// # Example
/// ```ignore
/// if let Some(backend) = ssbusync_guest::current_index_backend() {
///     skyline::println!("index backend: {:?}", backend);
/// }
/// ```
pub fn current_index_backend() -> Option<IndexBackend> {
    call_u32(SSBUSYNC_CURRENT_INDEX_BACKEND_SYMBOL).and_then(IndexBackend::from_u32)
}

/// Registers a raw `u32` callback for remote vsync-change notifications.
///
/// Pass `None` to unregister the callback.
///
/// # Example
/// ```ignore
/// extern "C" fn on_vsync_changed(raw: u32) {
///     skyline::println!("vsync enabled = {}", raw != 0);
/// }
///
/// let registered = ssbusync_guest::set_vsync_changed_callback(Some(on_vsync_changed));
/// ```
pub fn set_vsync_changed_callback(callback: Option<StateCallback>) -> Option<bool> {
    call_callback_reg(SSBUSYNC_SET_VSYNC_CHANGED_CALLBACK_SYMBOL, callback).map(|value| value != 0)
}

/// Clears the raw vsync-change callback.
///
/// # Example
/// ```ignore
/// let cleared = ssbusync_guest::clear_vsync_changed_callback();
/// ```
pub fn clear_vsync_changed_callback() -> Option<bool> {
    set_vsync_changed_callback(None)
}

/// Registers a raw `u32` callback for remote buffer-mode changes.
///
/// Pass `None` to unregister the callback.
///
/// # Example
/// ```ignore
/// extern "C" fn on_buffer_mode_changed(raw: u32) {
///     skyline::println!("buffer mode changed to raw value {raw}");
/// }
///
/// let registered =
///     ssbusync_guest::set_buffer_mode_changed_callback(Some(on_buffer_mode_changed));
/// ```
pub fn set_buffer_mode_changed_callback(callback: Option<StateCallback>) -> Option<bool> {
    call_callback_reg(SSBUSYNC_SET_BUFFER_MODE_CHANGED_CALLBACK_SYMBOL, callback)
        .map(|value| value != 0)
}

/// Clears the raw buffer-mode callback.
///
/// # Example
/// ```ignore
/// let cleared = ssbusync_guest::clear_buffer_mode_changed_callback();
/// ```
pub fn clear_buffer_mode_changed_callback() -> Option<bool> {
    set_buffer_mode_changed_callback(None)
}

/// Registers a raw `u32` callback for remote index-backend changes.
///
/// Pass `None` to unregister the callback.
///
/// # Example
/// ```ignore
/// extern "C" fn on_index_backend_changed(raw: u32) {
///     skyline::println!("index backend changed to raw value {raw}");
/// }
///
/// let registered =
///     ssbusync_guest::set_index_backend_changed_callback(Some(on_index_backend_changed));
/// ```
pub fn set_index_backend_changed_callback(callback: Option<StateCallback>) -> Option<bool> {
    call_callback_reg(SSBUSYNC_SET_INDEX_BACKEND_CHANGED_CALLBACK_SYMBOL, callback)
        .map(|value| value != 0)
}

/// Clears the raw index-backend callback.
///
/// # Example
/// ```ignore
/// let cleared = ssbusync_guest::clear_index_backend_changed_callback();
/// ```
pub fn clear_index_backend_changed_callback() -> Option<bool> {
    set_index_backend_changed_callback(None)
}
