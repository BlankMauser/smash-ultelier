use core::sync::atomic::{AtomicU32, Ordering};

/// Shared event written by a producer and sampled by a consumer.
///
/// The low 8 bits store the current mode value. The upper bits are a counter
/// for detecting event changes.
#[repr(C)]
pub struct SharedState {
    pub packed: AtomicU32,
}

/// Lightweight event publisher for mode changes.
///
/// Publishing every frame is fine;
pub struct Publisher {
    local_packed: u32,
}

impl Publisher {
    #[inline(always)]
    pub const fn new(mode: u8) -> Self {
        Self { local_packed: mode as u32 }
    }

    #[inline(always)]
    pub fn set_mode(&mut self, mode: u8) {
        self.local_packed = (self.local_packed & !0xFF) | mode as u32;
    }

    /// Publishes the current mode and bumps the sequence counter.
    ///
    /// This can be called every frame.
    #[inline(always)]
    pub fn tick(&mut self, shared: &SharedState) {
        self.local_packed = self.local_packed.wrapping_add(1 << 8);
        shared.packed.store(self.local_packed, Ordering::Relaxed);
    }
}
