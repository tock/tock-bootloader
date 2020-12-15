//! Decide to enter bootloader based on special RAM location.
//!
//! On the nRF52 the GPREGRET memory location is preserved on a soft reset. This
//! allows the kernel to set this before resetting and resume in the bootloader.

const DFU_MAGIC_SERIAL_ONLY_RESET: u32 = 0x4e;

pub struct BootloaderEntryGpRegRet {
    nrf_power: &'static nrf52::power::Power<'static>,
}

impl BootloaderEntryGpRegRet {
    pub fn new(nrf_power: &'static nrf52::power::Power<'static>) -> BootloaderEntryGpRegRet {
        BootloaderEntryGpRegRet { nrf_power }
    }
}

impl bootloader::interfaces::BootloaderEntry for BootloaderEntryGpRegRet {
    fn stay_in_bootloader(&self) -> bool {
        // Check if the retention flag matches the special variable indicating
        // we should stay in the bootloader.
        self.nrf_power.get_gpregret() == DFU_MAGIC_SERIAL_ONLY_RESET
    }
}
