//! Decide to enter bootloader based on checking for rapid double resets.

use kernel::utilities::cells::VolatileCell;
use kernel::utilities::StaticRef;

/// Magic value for the double reset memory location indicating we should stay
/// in the bootloader. This value (and name) is taken from the Adafruit nRF52
/// bootloader.
const DFU_DBL_RESET_MAGIC: u32 = 0x5A1AD5;

/// Memory location we use as a flag for detecting a double reset.
///
/// I have no idea why we use address 0x20007F7C, but that is what the Adafruit
/// nRF52 bootloader uses, so I copied it.
const DOUBLE_RESET_MEMORY_LOCATION: StaticRef<VolatileCell<u32>> =
    unsafe { StaticRef::new(0x20007F7C as *const VolatileCell<u32>) };

pub struct BootloaderEntryDoubleReset {
    double_reset: StaticRef<VolatileCell<u32>>,
}

impl BootloaderEntryDoubleReset {
    pub fn new() -> BootloaderEntryDoubleReset {
        BootloaderEntryDoubleReset {
            double_reset: DOUBLE_RESET_MEMORY_LOCATION,
        }
    }
}

impl bootloader::interfaces::BootloaderEntry for BootloaderEntryDoubleReset {
    fn stay_in_bootloader(&self) -> bool {
        // If the retention flag is not set, then we check for the double reset
        // memory location. If this is set to a magic value, then we got two
        // resets in a short amount of time and we want to go into the
        // bootloader.
        if self.double_reset.get() == DFU_DBL_RESET_MAGIC {
            self.double_reset.set(0);
            return true;
        }

        // If neither magic value is set, then we need to check if we just got
        // the first of a double reset. We do this by setting our flag and
        // entering a busy loop. If the busy loop finishes then we must not have
        // gotten a second reset and we go to the kernel. If the busy loop
        // doesn't finish because we got a reset in the middle, then the
        // bootloader will restart and the check above should trigger.
        self.double_reset.set(DFU_DBL_RESET_MAGIC);
        for _ in 0..2000000 {
            cortexm4::support::nop();
        }
        self.double_reset.set(0);

        // Default to jumping out of the bootloader.
        false
    }
}
