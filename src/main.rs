
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
    // PB4 默认作为 JTAG 引脚，无法拉低，需要配置为只采用 SWD 模式
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