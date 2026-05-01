#[cfg(feature = "plugin")]
pub mod common;
#[cfg(feature = "plugin")]
pub mod console;

#[cfg(feature = "sync-guest")]
pub mod sync_guest;

#[cfg(feature = "plugin")]
#[link(name = "imgui_smash")]
unsafe extern "C" {}

#[cfg(feature = "plugin")]
fn panic_hook() {
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

#[cfg(feature = "plugin")]
#[skyline::main(name = "ultelier")]
pub fn main() {
    panic_hook();
    sync_guest::runtime::install_auto_profile_switcher();
    console::install();
}
