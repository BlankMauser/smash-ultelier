use crate::profile::{self, DockedProfile};
use skyline::hooks::InlineCtx;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const CSS_CURRENT_PLAYER_COUNT_OFFSET: usize = 0x160;
const DEFAULT_MATCH_PLAYER_COUNT: u32 = 2;

static INSTALLED: AtomicBool = AtomicBool::new(false);
static IN_MATCH: AtomicBool = AtomicBool::new(false);
static LAST_CSS_PLAYER_COUNT: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" {
    #[link_name = "\u{1}_ZN3app7fighter23get_fighter_entry_countEv"]
    fn get_fighter_entry_count() -> u32;

    #[link_name = "\u{1}_ZN3app9smashball16is_training_modeEv"]
    fn is_training_mode() -> bool;
}

#[inline]
fn clamp_player_count(value: u32) -> u32 {
    value.clamp(1, 8)
}

#[inline]
fn observed_player_count() -> u32 {
    let css_count = LAST_CSS_PLAYER_COUNT.load(Ordering::Acquire);
    if css_count != 0 {
        return css_count;
    }

    let fighter_count = unsafe { get_fighter_entry_count() };
    if fighter_count == 0 {
        DEFAULT_MATCH_PLAYER_COUNT
    } else {
        clamp_player_count(fighter_count)
    }
}

#[inline]
fn classify_active_match() -> DockedProfile {
    if unsafe { is_training_mode() } {
        return DockedProfile::Singles;
    }

    if observed_player_count() >= 4 {
        DockedProfile::Ffa
    } else {
        DockedProfile::Singles
    }
}

/// Returns the docked-profile state implied by the currently observed game
/// runtime.
///
/// Outside a match this returns `DockedProfile::Rest`. During a match it
/// classifies training and 1v1 as `Singles`, and 4+ players as `Ffa`.
///
/// # Example
/// ```ignore
/// use ssbusync_guest::runtime;
///
/// let state = runtime::current_runtime_profile();
/// skyline::println!("runtime state = {:?}", state);
/// ```
pub fn current_runtime_profile() -> DockedProfile {
    if !IN_MATCH.load(Ordering::Acquire) {
        DockedProfile::Rest
    } else {
        classify_active_match()
    }
}

fn apply_runtime_profile() {
    let desired = current_runtime_profile();
    let _ = profile::apply_docked_profile(desired);
}

#[inline]
fn store_css_player_count(raw: u32) {
    LAST_CSS_PLAYER_COUNT.store(clamp_player_count(raw), Ordering::Release);
}

#[skyline::hook(offset = 0x1a26200)]
unsafe fn css_player_count_changed(param_1: i64, prev_num: i32, changed_by_player: u32) {
    call_original!(param_1, prev_num, changed_by_player);
    let count = *((param_1 + CSS_CURRENT_PLAYER_COUNT_OFFSET as i64) as *const u32);
    store_css_player_count(count);
    IN_MATCH.store(false, Ordering::Release);
    let _ = profile::apply_rest();
}

#[skyline::hook(offset = 0x1345558, inline)]
unsafe fn on_match_start(_: &InlineCtx) {
    IN_MATCH.store(true, Ordering::Release);
    apply_runtime_profile();
}

#[skyline::hook(offset = 0x1d68b94, inline)]
unsafe fn on_match_end(_: &InlineCtx) {
    IN_MATCH.store(false, Ordering::Release);
    let _ = profile::apply_rest();
}

#[skyline::hook(offset = 0x235a650, inline)]
unsafe fn on_main_menu(_: &InlineCtx) {
    IN_MATCH.store(false, Ordering::Release);
    let _ = profile::apply_rest();
}

/// Installs Skyline hooks that automatically map menu, singles, and FFA game
/// states onto the configured docked-profile table.
///
/// Call this once during plugin startup. Repeated calls are ignored.
///
/// # Example
/// ```ignore
/// #[skyline::main(name = "my_plugin")]
/// pub fn main() {
///     ssbusync_guest::runtime::install_auto_profile_switcher();
/// }
/// ```
pub fn install_auto_profile_switcher() {
    if INSTALLED.swap(true, Ordering::AcqRel) {
        return;
    }

    let _ = profile::sync_from_remote();

    skyline::install_hooks!(
        css_player_count_changed,
        on_match_start,
        on_match_end,
        on_main_menu
    );
}
