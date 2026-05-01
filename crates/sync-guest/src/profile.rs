use crate::callback::Callback;
use crate::OverclockProfile;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

pub type DockedProfileChangedCallback = extern "C" fn(DockedProfile);

const UNKNOWN_STATE: u32 = u32::MAX;
const UNKNOWN_PROFILE: u32 = u32::MAX;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockedProfile {
    Rest = 0,
    Singles = 1,
    Ffa = 2,
}

impl DockedProfile {
    /// Returns every supported docked-profile state in display order.
    ///
    /// # Example
    /// ```rust
    /// use ultelier::sync_guest::profile::DockedProfile;
    ///
    /// assert_eq!(
    ///     DockedProfile::all(),
    ///     [DockedProfile::Rest, DockedProfile::Singles, DockedProfile::Ffa]
    /// );
    /// ```
    pub const fn all() -> [Self; 3] {
        [Self::Rest, Self::Singles, Self::Ffa]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockedProfileMap {
    pub rest: OverclockProfile,
    pub singles: OverclockProfile,
    pub ffa: OverclockProfile,
}

impl DockedProfileMap {
    /// Looks up the overclock profile currently assigned to a docked state.
    ///
    /// # Example
    /// ```rust
    /// use ultelier::sync_guest::profile::{DockedProfile, DockedProfileMap};
    /// use ultelier::sync_guest::OverclockProfile;
    ///
    /// let map = DockedProfileMap::default();
    /// assert_eq!(
    ///     map.profile_for(DockedProfile::Singles),
    ///     OverclockProfile::PerformanceSingles
    /// );
    /// ```
    pub const fn profile_for(self, state: DockedProfile) -> OverclockProfile {
        match state {
            DockedProfile::Rest => self.rest,
            DockedProfile::Singles => self.singles,
            DockedProfile::Ffa => self.ffa,
        }
    }

    /// Finds which docked state currently maps to the given overclock profile.
    ///
    /// Returns `None` when the profile is not present in the map.
    ///
    /// # Example
    /// ```rust
    /// use ultelier::sync_guest::profile::{DockedProfile, DockedProfileMap};
    /// use ultelier::sync_guest::OverclockProfile;
    ///
    /// let map = DockedProfileMap::default();
    /// assert_eq!(
    ///     map.state_for(OverclockProfile::PerformanceFfa),
    ///     Some(DockedProfile::Ffa)
    /// );
    /// ```
    pub const fn state_for(self, profile: OverclockProfile) -> Option<DockedProfile> {
        if profile as u32 == self.rest as u32 {
            Some(DockedProfile::Rest)
        } else if profile as u32 == self.singles as u32 {
            Some(DockedProfile::Singles)
        } else if profile as u32 == self.ffa as u32 {
            Some(DockedProfile::Ffa)
        } else {
            None
        }
    }

    /// Reassigns one docked state and returns the previous profile.
    ///
    /// # Example
    /// ```rust
    /// use ultelier::sync_guest::profile::{DockedProfile, DockedProfileMap};
    /// use ultelier::sync_guest::OverclockProfile;
    ///
    /// let mut map = DockedProfileMap::default();
    /// let previous = map.set(DockedProfile::Rest, OverclockProfile::PerformanceSingles);
    ///
    /// assert_eq!(previous, OverclockProfile::Rest);
    /// assert_eq!(
    ///     map.profile_for(DockedProfile::Rest),
    ///     OverclockProfile::PerformanceSingles
    /// );
    /// ```
    pub fn set(&mut self, state: DockedProfile, profile: OverclockProfile) -> OverclockProfile {
        let slot = match state {
            DockedProfile::Rest => &mut self.rest,
            DockedProfile::Singles => &mut self.singles,
            DockedProfile::Ffa => &mut self.ffa,
        };
        core::mem::replace(slot, profile)
    }
}

impl Default for DockedProfileMap {
    fn default() -> Self {
        Self {
            rest: OverclockProfile::Rest,
            singles: OverclockProfile::PerformanceSingles,
            ffa: OverclockProfile::PerformanceFfa,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyResult {
    Unchanged,
    Applied,
    Rejected,
    RemoteUnavailable,
}

static PROFILE_MAP: Mutex<DockedProfileMap> = Mutex::new(DockedProfileMap {
    rest: OverclockProfile::Rest,
    singles: OverclockProfile::PerformanceSingles,
    ffa: OverclockProfile::PerformanceFfa,
});
static CHANGED_CALLBACK: Mutex<Callback<DockedProfileChangedCallback>> =
    Mutex::new(Callback::new(None));
static APPLY_LOCK: Mutex<()> = Mutex::new(());
static CURRENT_STATE: AtomicU32 = AtomicU32::new(UNKNOWN_STATE);
static CURRENT_PROFILE: AtomicU32 = AtomicU32::new(UNKNOWN_PROFILE);

fn with_profile_map<R>(f: impl FnOnce(&mut DockedProfileMap) -> R) -> R {
    let mut map = PROFILE_MAP.lock().unwrap_or_else(|err| err.into_inner());
    f(&mut map)
}

fn with_changed_callback<R>(f: impl FnOnce(&mut Callback<DockedProfileChangedCallback>) -> R) -> R {
    let mut callback = CHANGED_CALLBACK
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    f(&mut callback)
}

fn cache_state(state: Option<DockedProfile>, profile: Option<OverclockProfile>) {
    CURRENT_STATE.store(
        state.map(|value| value as u32).unwrap_or(UNKNOWN_STATE),
        Ordering::Release,
    );
    CURRENT_PROFILE.store(
        profile.map(|value| value as u32).unwrap_or(UNKNOWN_PROFILE),
        Ordering::Release,
    );
}

fn cached_state_matches(state: DockedProfile, profile: OverclockProfile) -> bool {
    CURRENT_STATE.load(Ordering::Acquire) == state as u32
        && CURRENT_PROFILE.load(Ordering::Acquire) == profile as u32
}

fn invoke_changed_callback(state: DockedProfile) {
    with_changed_callback(|callback| {
        let _ = callback.invoke((state,));
    });
}

/// Returns the current local docked-profile mapping table.
///
/// # Example
/// ```ignore
/// use ultelier::sync_guest::profile::{self, DockedProfile};
///
/// let map = profile::docked_profile_map();
/// let singles_profile = map.profile_for(DockedProfile::Singles);
/// ```
pub fn docked_profile_map() -> DockedProfileMap {
    with_profile_map(|map| *map)
}

/// Replaces the full local docked-profile mapping table.
///
/// This only updates the guest-side mapping. Call `apply_docked_profile` to
/// push one of the mapped states to the remote runtime.
///
/// # Example
/// ```ignore
/// use ultelier::sync_guest::profile::{self, DockedProfileMap};
/// use ultelier::sync_guest::OverclockProfile;
///
/// profile::set_docked_profile_map(DockedProfileMap {
///     rest: OverclockProfile::Rest,
///     singles: OverclockProfile::PerformanceSingles,
///     ffa: OverclockProfile::PerformanceFfa,
/// });
/// ```
pub fn set_docked_profile_map(map: DockedProfileMap) {
    with_profile_map(|slot| *slot = map);
    invalidate_cache();
}

/// Reassigns one docked-profile entry and returns the previous profile.
///
/// # Example
/// ```ignore
/// use ultelier::sync_guest::profile::{self, DockedProfile};
/// use ultelier::sync_guest::OverclockProfile;
///
/// let previous = profile::set_docked_profile(
///     DockedProfile::Rest,
///     OverclockProfile::Rest,
/// );
/// let _ = previous;
/// ```
pub fn set_docked_profile(state: DockedProfile, profile: OverclockProfile) -> OverclockProfile {
    let previous = with_profile_map(|map| map.set(state, profile));
    invalidate_cache();
    previous
}

/// Registers a callback that runs after `apply_docked_profile` changes state.
///
/// # Example
/// ```ignore
/// use ultelier::sync_guest::profile::{self, DockedProfile};
///
/// extern "C" fn on_profile_changed(state: DockedProfile) {
///     skyline::println!("docked profile changed to {:?}", state);
/// }
///
/// let previous = profile::set_docked_profile_changed_callback(on_profile_changed);
/// ```
pub fn set_docked_profile_changed_callback(
    callback: DockedProfileChangedCallback,
) -> Option<DockedProfileChangedCallback> {
    with_changed_callback(|slot| slot.set(callback))
}

/// Clears the docked-profile changed callback.
///
/// # Example
/// ```ignore
/// let previous = ultelier::sync_guest::profile::clear_docked_profile_changed_callback();
/// ```
pub fn clear_docked_profile_changed_callback() -> Option<DockedProfileChangedCallback> {
    with_changed_callback(|slot| slot.clear())
}

/// Returns the last docked state cached by this crate.
///
/// Use `sync_from_remote` when you want to refresh that cache from the remote
/// runtime first.
///
/// # Example
/// ```ignore
/// let cached = ultelier::sync_guest::profile::cached_docked_profile();
/// ```
pub fn cached_docked_profile() -> Option<DockedProfile> {
    match CURRENT_STATE.load(Ordering::Acquire) {
        0 => Some(DockedProfile::Rest),
        1 => Some(DockedProfile::Singles),
        2 => Some(DockedProfile::Ffa),
        _ => None,
    }
}

/// Returns the last overclock profile cached by this crate.
///
/// # Example
/// ```ignore
/// let cached = ultelier::sync_guest::profile::cached_overclock_profile();
/// ```
pub fn cached_overclock_profile() -> Option<OverclockProfile> {
    OverclockProfile::from_u32(CURRENT_PROFILE.load(Ordering::Acquire))
}

/// Clears the cached docked-state and overclock-profile values.
///
/// # Example
/// ```ignore
/// use ultelier::sync_guest::profile;
///
/// profile::invalidate_cache();
/// assert_eq!(profile::cached_docked_profile(), None);
/// ```
pub fn invalidate_cache() {
    cache_state(None, None);
}

/// Refreshes the local cache from the currently active remote overclock profile.
///
/// The outer `Option` is `None` when the remote is unavailable. The inner
/// `Option` is `None` when the remote profile does not map to a known
/// `OverclockProfile`.
///
/// # Example
/// ```ignore
/// let remote = ultelier::sync_guest::profile::sync_from_remote();
/// ```
pub fn sync_from_remote() -> Option<Option<OverclockProfile>> {
    let result = crate::current_overclock_profile()?;
    let state = result.and_then(|profile| docked_profile_map().state_for(profile));
    cache_state(state, result);
    Some(result)
}

/// Applies the overclock profile mapped to the requested docked state.
///
/// This is the main entry point when game code wants to say "treat the current
/// situation as rest, singles, or FFA" and let the mapping table choose the
/// actual remote profile.
///
/// # Example
/// ```ignore
/// use ultelier::sync_guest::profile::{self, ApplyResult, DockedProfile};
///
/// match profile::apply_docked_profile(DockedProfile::Singles) {
///     ApplyResult::Applied | ApplyResult::Unchanged => {}
///     ApplyResult::Rejected => skyline::println!("remote rejected singles profile"),
///     ApplyResult::RemoteUnavailable => skyline::println!("ssbusync is unavailable"),
/// }
/// ```
pub fn apply_docked_profile(state: DockedProfile) -> ApplyResult {
    let mapped = docked_profile_map().profile_for(state);
    if cached_state_matches(state, mapped) {
        return ApplyResult::Unchanged;
    }

    let _guard = APPLY_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let mapped = docked_profile_map().profile_for(state);
    if cached_state_matches(state, mapped) {
        return ApplyResult::Unchanged;
    }

    if let Some(remote) = sync_from_remote() {
        if remote == Some(mapped) && cached_state_matches(state, mapped) {
            return ApplyResult::Unchanged;
        }
    }

    match crate::set_overclock_profile(mapped) {
        Some(true) => {
            cache_state(Some(state), Some(mapped));
            invoke_changed_callback(state);
            ApplyResult::Applied
        }
        Some(false) => ApplyResult::Rejected,
        None => ApplyResult::RemoteUnavailable,
    }
}

/// Convenience wrapper for `apply_docked_profile(DockedProfile::Rest)`.
///
/// # Example
/// ```ignore
/// let result = ultelier::sync_guest::profile::apply_rest();
/// ```
pub fn apply_rest() -> ApplyResult {
    apply_docked_profile(DockedProfile::Rest)
}

/// Convenience wrapper for `apply_docked_profile(DockedProfile::Singles)`.
///
/// # Example
/// ```ignore
/// let result = ultelier::sync_guest::profile::apply_singles();
/// ```
pub fn apply_singles() -> ApplyResult {
    apply_docked_profile(DockedProfile::Singles)
}

/// Convenience wrapper for `apply_docked_profile(DockedProfile::Ffa)`.
///
/// # Example
/// ```ignore
/// let result = ultelier::sync_guest::profile::apply_ffa();
/// ```
pub fn apply_ffa() -> ApplyResult {
    apply_docked_profile(DockedProfile::Ffa)
}
