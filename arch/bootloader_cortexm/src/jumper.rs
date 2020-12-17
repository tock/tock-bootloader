pub struct CortexMJumper {}

impl CortexMJumper {
    pub fn new() -> CortexMJumper {
        CortexMJumper {}
    }
}

impl bootloader::interfaces::Jumper for CortexMJumper {
    fn jump(&self, address: u32) -> ! {
        unsafe {
            asm!(
            ".syntax unified                                                                                   \n\
            mov r0, {0}         // The address of the payload's .vectors                                       \n\
            ldr r1, =0xe000ed08 // The address of the VTOR register (0xE000E000(SCS) + 0xD00(SCB) + 0x8(VTOR)) \n\
            str r0, [r1]        // Move the payload's VT address into the VTOR register                        \n\
            ldr r1, [r0]        // Move the payload's initial SP into r1                                       \n\
            mov sp, r1          // Set our SP to that                                                          \n\
            ldr r0, [r0, #4]    // Load the payload's ENTRY into r0                                            \n\
            bx  r0              // Whoopee",
            in(reg) address,
            );
        }
        loop {}
    }
}
