//! Helpers for storing optional `extern "C"` callbacks and invoking them later.
//!
//! # Examples
//!
//! Store and invoke a callback locally:
//!
//! ```ignore
//! use ultelier::common::callback::Callback;
//!
//! extern "C" fn on_state_changed(value: u32) {
//!     assert_eq!(value, 1);
//! }
//!
//! let callback = Callback::from_fn(on_state_changed as extern "C" fn(u32));
//! assert!(callback.is_set());
//! assert_eq!(callback.invoke((1,)), Some(()));
//! ```
//!
//! Subscribe to a typed `sync-guest` event and clear it later:
//!
//! ```ignore
//! use ultelier::sync_guest::{self, events};
//!
//! extern "C" fn on_vsync_changed(enabled: bool) {
//!     let _ = enabled;
//! }
//!
//! if sync_guest::remote_present() {
//!     let registered = events::set_typed_vsync_changed(on_vsync_changed);
//!     assert!(registered);
//!
//!     // Clear the remote registration when the callback is no longer needed.
//!     let _ = events::clear_typed_vsync_changed();
//! }
//! ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Callback<F> {
    func: Option<F>,
}

impl<F> Default for Callback<F> {
    fn default() -> Self {
        Self { func: None }
    }
}

impl<F: Copy> Callback<F> {
    pub const fn new(func: Option<F>) -> Self {
        Self { func }
    }

    pub const fn from_fn(func: F) -> Self {
        Self { func: Some(func) }
    }

    pub const fn is_set(&self) -> bool {
        self.func.is_some()
    }

    pub const fn get(&self) -> Option<F> {
        self.func
    }

    pub fn set(&mut self, func: F) -> Option<F> {
        self.replace(Some(func))
    }

    pub fn clear(&mut self) -> Option<F> {
        self.replace(None)
    }

    pub fn replace(&mut self, func: Option<F>) -> Option<F> {
        core::mem::replace(&mut self.func, func)
    }

    pub fn invoke<Args>(&self, args: Args) -> Option<<F as CallbackSignature<Args>>::Output>
    where
        F: CallbackSignature<Args>,
    {
        self.func.map(|func| func.call(args))
    }
}

pub trait CallbackSignature<Args> {
    type Output;

    fn call(self, args: Args) -> Self::Output;
}

macro_rules! impl_callback_signature {
    ($(($($ty:ident : $arg:ident),*)),+ $(,)?) => {
        $(
            impl<R, $($ty,)*> CallbackSignature<($($ty,)*)> for extern "C" fn($($ty),*) -> R {
                type Output = R;

                #[inline]
                fn call(self, ($($arg,)*): ($($ty,)*)) -> Self::Output {
                    self($($arg),*)
                }
            }
        )+
    };
}

impl_callback_signature! {
    (),
    (A0: a0),
    (A0: a0, A1: a1),
    (A0: a0, A1: a1, A2: a2),
    (A0: a0, A1: a1, A2: a2, A3: a3),
    (A0: a0, A1: a1, A2: a2, A3: a3, A4: a4)
}
