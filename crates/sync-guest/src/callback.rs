/// Small optional-callback wrapper used by the higher-level event helpers.
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
    /// Creates a callback wrapper from an optional function pointer.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::callback::Callback;
    ///
    /// extern "C" fn on_event(_: u32) {}
    ///
    /// let callback = Callback::new(Some(on_event));
    /// assert!(callback.is_set());
    /// ```
    pub const fn new(func: Option<F>) -> Self {
        Self { func }
    }

    /// Creates a callback wrapper that is already populated.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::callback::Callback;
    ///
    /// extern "C" fn on_event(_: u32) {}
    ///
    /// let callback = Callback::from_fn(on_event);
    /// assert_eq!(callback.get(), Some(on_event));
    /// ```
    pub const fn from_fn(func: F) -> Self {
        Self { func: Some(func) }
    }

    /// Returns `true` when a callback has been stored.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::callback::Callback;
    ///
    /// extern "C" fn on_event(_: u32) {}
    ///
    /// let empty = Callback::new(None::<extern "C" fn(u32)>);
    /// let set = Callback::from_fn(on_event);
    ///
    /// assert!(!empty.is_set());
    /// assert!(set.is_set());
    /// ```
    pub const fn is_set(&self) -> bool {
        self.func.is_some()
    }

    /// Returns the currently stored callback.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::callback::Callback;
    ///
    /// extern "C" fn on_event(_: u32) {}
    ///
    /// let callback = Callback::from_fn(on_event);
    /// assert_eq!(callback.get(), Some(on_event));
    /// ```
    pub const fn get(&self) -> Option<F> {
        self.func
    }

    /// Stores a callback and returns the previous one, if any.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::callback::Callback;
    ///
    /// extern "C" fn first(_: u32) {}
    /// extern "C" fn second(_: u32) {}
    ///
    /// let mut callback = Callback::from_fn(first);
    /// assert_eq!(callback.set(second), Some(first));
    /// assert_eq!(callback.get(), Some(second));
    /// ```
    pub fn set(&mut self, func: F) -> Option<F> {
        self.replace(Some(func))
    }

    /// Clears the stored callback and returns the previous one.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::callback::Callback;
    ///
    /// extern "C" fn on_event(_: u32) {}
    ///
    /// let mut callback = Callback::from_fn(on_event);
    /// assert_eq!(callback.clear(), Some(on_event));
    /// assert!(!callback.is_set());
    /// ```
    pub fn clear(&mut self) -> Option<F> {
        self.replace(None)
    }

    /// Replaces the stored callback with an optional value.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::callback::Callback;
    ///
    /// extern "C" fn on_event(_: u32) {}
    ///
    /// let mut callback = Callback::new(None::<extern "C" fn(u32)>);
    /// assert_eq!(callback.replace(Some(on_event)), None);
    /// assert_eq!(callback.replace(None), Some(on_event));
    /// ```
    pub fn replace(&mut self, func: Option<F>) -> Option<F> {
        core::mem::replace(&mut self.func, func)
    }

    /// Invokes the stored callback if one is present.
    ///
    /// # Example
    /// ```rust
    /// use ssbusync_guest::callback::Callback;
    ///
    /// extern "C" fn add_one(value: u32) -> u32 {
    ///     value + 1
    /// }
    ///
    /// let callback = Callback::from_fn(add_one);
    /// assert_eq!(callback.invoke((41,)), Some(42));
    /// ```
    pub fn invoke<Args>(&self, args: Args) -> Option<<F as CallbackSignature<Args>>::Output>
    where
        F: CallbackSignature<Args>,
    {
        self.func.map(|func| func.call(args))
    }
}

/// Trait implemented for supported `extern "C"` callback signatures so
/// `Callback::invoke` can call them in a typed way.
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
