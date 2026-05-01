#[cfg(feature = "console")]
pub mod common;
#[cfg(feature = "console")]
pub mod console;

#[cfg(feature = "sync-guest")]
pub use ssbusync_guest as sync_guest;

#[cfg(feature = "console")]
#[link(name = "imgui_smash")]
unsafe extern "C" {}

pub fn panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let err_msg = format!("ultelier panicked at '{}', {}", msg, location);
        skyline::error::show_error(
            69,
            "ultelier has panicked. Please open the details and send a screenshot to the developer, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));
}

#[cfg(feature = "no-dep")]
#[skyline::main(name = "ultelier")]
pub fn main() {
    panic_hook();
    #[cfg(feature = "auto-profile-switcher")]
    sync_guest::runtime::install_auto_profile_switcher();
    #[cfg(feature = "console")]
    console::install();
}
