//! Trait definitions for the bootloader.

/// Trait for implementing the decision logic on whether to run the bootloader
/// or jump to application code.
pub trait BootloaderEntry {
    /// Called to check if the bootloader should stay running (i.e. enter the
    /// bootloader).
    ///
    /// Returns `true` if we should stay in the bootloader, or `false` to jump
    /// to application code.
    fn stay_in_bootloader(&self) -> bool;
}

/// Trait for handling the jump from the bootloader to the kernel.
pub trait Jumper {
    /// Jump execution to the specified address as though the chip had started
    /// executing there.
    fn jump(&self, address: u32) -> !;
}
