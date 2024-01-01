
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
    let dp = pac::Peripherals::take().unwrap();

    dp.RCC.apb1enr().write(|w| w
        .pwren().set_bit()
    );

    dp.RCC.apb2enr().write(|w| w
        .iopben().set_bit()
        .afioen().set_bit()
    );

    // PB4 默认作为 JTAG 引脚，无法拉低，需要配置为只采用 SWD 模式
    dp.AFIO.mapr().write(|w| unsafe { w.
        swj_cfg().bits(0b010)
    });

    dp.GPIOB.crl().write(|w| unsafe { w
        .mode4().bits(0b01)
        .cnf4().bits(0b00)
    });
    dp.GPIOB.odr().write(|w| w
        .odr4().clear_bit()
    );

    let mut i: u32 = 0;
    loop {
        while i <= 100_0000 {
            i += 1;
        }
        i = 0;
        dp.GPIOB.odr().modify(|r, w| w
            .odr4().bit(!r.odr4().bit())
        );
    }
}