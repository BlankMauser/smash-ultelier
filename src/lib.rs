#[cfg(feature = "console")]
pub mod common;
#[cfg(feature = "console")]
pub mod console;

#[cfg(feature = "sync-guest")]
pub use ssbusync_guest as sync_guest;

#[cfg(feature = "console")]
#[link(name = "imgui_smash")]
unsafe extern "C" {}

#[cfg(feature = "sync-guest")]
fn buffer_mode_name(mode: sync_guest::BufferMode) -> &'static str {
    match mode {
        sync_guest::BufferMode::Double => "double",
        sync_guest::BufferMode::Triple => "triple",
    }
}

#[cfg(feature = "sync-guest")]
extern "C" fn on_buffer_mode_changed(mode: sync_guest::BufferMode) {
    skyline::println!(
        "[ultelier] ssbusync buffer mode changed -> {}",
        buffer_mode_name(mode)
    );
}

#[cfg(feature = "sync-guest")]
fn install_buffer_mode_logger() {
    let current_mode = sync_guest::env_flags().map(|flags| {
        if flags.contains(sync_guest::EnvironmentFlags::TRIPLE_ENABLED) {
            sync_guest::BufferMode::Triple
        } else {
            sync_guest::BufferMode::Double
        }
    });

    match current_mode {
        Some(mode) => skyline::println!(
            "[ultelier] ssbusync buffer mode at startup -> {}",
            buffer_mode_name(mode)
        ),
        None => skyline::println!("[ultelier] ssbusync symbols unavailable at startup"),
    }

    let registered = sync_guest::events::set_typed_buffer_mode_changed(on_buffer_mode_changed);
    skyline::println!("[ultelier] buffer mode callback registered={registered}");
}

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
    #[cfg(feature = "sync-guest")]
    install_buffer_mode_logger();
    #[cfg(feature = "auto-profile-switcher")]
    sync_guest::runtime::install_auto_profile_switcher();
    #[cfg(feature = "console")]
    console::install();
}
