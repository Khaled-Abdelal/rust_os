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
/*
* Rust has it's own testing framework but it depends on the standard library
* we use the custom_test_frameworks feature to define our own test runner
* */
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
/*
* The custom_test_framewrok feature generates it's own main function that calls the test runner
* we need to specify a custom name for the generated function and then call it our self in the
* _start function (our entry point)
* */
#![reexport_test_harness_main = "test_main"]
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
    // panic!("Some panic");
    println!("Hello World{}", "!");

    // call the test runner if compiling for tests
    #[cfg(test)]
    test_main();

    loop {}
}

/*
* The standard library defines a panic handler but without it we need to define our own
* the ! return type means that this function never returns (it is a DIVERGING_FUNCTION)
* */
use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// Define a module to print things to the screen through VGA text buffer
mod vga_buffer;

// a custom test runner
#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}

/*
* After running the tests we need a way to exit
* we can send an exit instruction to QEMU to terminate the machine
* QEMU supports a special isa-debug-exit device, which provides an easy way to exit QEMU from the guest system
* isa-debug-exit uses a port mapped I/O interface
* we use the x86_64 crate to write to the port
* 0xf4 is the iobase of the isa-debug-exit device.
* */

// The actual exit codes don’t matter much, as long as they don’t clash with the default exit codes of QEMU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
