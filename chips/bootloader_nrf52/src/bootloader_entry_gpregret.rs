//! Decide to enter bootloader based on special RAM location.
//!
//! On the nRF52 the GPREGRET memory location is preserved on a soft reset. This
//! allows the kernel to set this before resetting and resume in the bootloader.

use kernel::common::cells::VolatileCell;
use kernel::common::StaticRef;

/// Magic value for the GPREGRET register that tells the Adafruit bootloader to
/// stay in bootloader mode. This value (and name) is taken from the Adafruit
/// nRF52 bootloader.
const DFU_MAGIC_SERIAL_ONLY_RESET: u32 = 0x4e;

/// Magic value for the GPREGRET register that tells our bootloader to stay in
/// bootloader mode. This value is based on the Adafruit nRF52 bootloader.
const DFU_MAGIC_TOCK_BOOTLOADER: u32 = 0x99;

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

pub struct BootloaderEntryGpRegRet {
    nrf_power: &'static nrf52::power::Power<'static>,
    double_reset: StaticRef<VolatileCell<u32>>,
}

impl BootloaderEntryGpRegRet {
    pub fn new(nrf_power: &'static nrf52::power::Power<'static>) -> BootloaderEntryGpRegRet {
        BootloaderEntryGpRegRet {
            nrf_power,
            double_reset: DOUBLE_RESET_MEMORY_LOCATION,
        }
    }
}

impl bootloader::interfaces::BootloaderEntry for BootloaderEntryGpRegRet {
    fn stay_in_bootloader(&self) -> bool {
        // Check if the retention flag matches the special variable indicating
        // we should stay in the bootloader. This would be set by the kernel
        // before doing a reset to indicate we should reboot into the
        // bootloader.
        if self.nrf_power.get_gpregret() == DFU_MAGIC_TOCK_BOOTLOADER {
            // Clear flag so we do not get stuck in the bootloader.
            self.nrf_power.set_gpregret(0);

            return true;
        }

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

        // Set this register to the value the Adafruit bootloader expects so that
        // the adafruit bootloader will stay in bootloader mode.
        //
        // There are three possible cases:
        //
        // 1. The adafruit bootloader does not exist. This is harmless in that
        //    case.
        // 2. The adafruit bootloader exists, but this bootloader is not jumping
        //    to the start address of that bootloader. In that case, this is
        //    still harmless.
        // 3. The adafruit bootloader exists, and we might jump to it (if we are
        //    not staying in this bootloader). In that case, setting this will
        //    allow us to stay in the adafruit bootloader.
        self.nrf_power.set_gpregret(DFU_MAGIC_SERIAL_ONLY_RESET);

        // Default to jumping out of the bootloader.
        false
    }
}
