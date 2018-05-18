
    .syntax unified
    .section .text.jumpfunc
    .global jump_into_user_code
    .thumb_func
jump_into_user_code:
    ldr r0, =0x10000 //The address of the payload's .vectors
    ldr r1, =0xe000ed08 //The address of the VTOR register (0xE000E000(SCS) + 0xD00(SCB) + 0x8(VTOR))
    str r0, [r1] //Move the payload's VT address into the VTOR register
    ldr r1, [r0] //Move the payload's initial SP into r1
    mov sp, r1 //Set our SP to that 
    ldr r0, [r0, #4] //Load the payload's ENTRY into r0
    bx  r0 //Whoopee
