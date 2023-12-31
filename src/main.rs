
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::sync::atomic;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        // TODO ?
        atomic::compiler_fence(atomic::Ordering::SeqCst);
    }
}

use stm32f103::{self as pac};
// use cortex_m;

#[cortex_m_rt::entry]
fn main() -> ! {
    // let mut cp = cortex_m::Peripherals::take().unwrap();
    // cp.SYST.disable_interrupt();

    let dp = pac::Peripherals::take().unwrap();

    dp.RCC.apb2enr().write(|w| w
        .iopben().set_bit()
    );

    dp.GPIOB.crl().write(|w| unsafe { w
        .mode4().bits(0b01)
        .cnf4().bits(0b00)
    });
    dp.GPIOB.odr().write(|w| w
        .odr4().clear_bit()
    );

    loop {}
}