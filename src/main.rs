/*
* Since this is an os we can't use any of standard library features that uses system calls
* the #[no_std] attribute is used to disable the standard library
* */
#![no_std]
/*
* In normal execution the first thing that is called is a c library called crt0 (C RUNTIME 0)
* that sets up the environment for the program to run which includes creating the stack and placing
* the arguments in the right registers. Then it calls the entry point of the RUST_RUNTIME
* defined with the start LANGUAGE_ITEM, the run time sets up stackoverflow guards etc...
* then finally the runtime calls the main function
*
* We need to define our own entry point, overriding the start language item won't work because
* there is no crt0 to call it
* */
#![no_main]

// this forces the compilar to not mangle the name of this function aka give it a
// random cryptic name ex: asdfaasdf  to avoid conflicts
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // extern "C" tells the compiler to use the C CALLING_CONVENTION for this function
    // _start is the entry point for most systems
    // ! return type means that this function never returns (it is a DIVERGING_FUNCTION)
    // that makes since because an OS is not called by another function but by a bootloader
    // so it should never return and instead it should invoke the EXIT_SYSCALL to terminate the OS
    // (shutdown the machine)
    loop {}
}

/*
* The standard library defines a panic handler but without it we need to define our own
* the ! return type means that this function never returns (it is a DIVERGING_FUNCTION)
* */
use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
