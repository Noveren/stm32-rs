
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

use stm32f103 as pac;

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();


    // Reset of all peripherals, Initializes the Flash interface and the Systick.
    dp.RCC.apb1enr().write(|w| w
        .pwren().set_bit()
    );
    dp.RCC.apb2enr().write(|w| w
        .afioen().set_bit()
    );

    // TODO NVIC

    // PB4 默认作为 JTAG 引脚，无法拉低，需要配置为只采用 SWD 模式
    dp.AFIO.mapr().write(|w| unsafe { w.
        swj_cfg().bits(0b010)
    });

    // SystemClock_Config
    {
        // dp.FLASH.acr().write(|w| w
        //     .latency().variant(0b010)
        // );
        // while dp.FLASH.acr().read().latency().bits() != 0b010 {}

        // dp.RCC.cr().write(|w| w
        //     .hseon().set_bit()
        // );
        // while !dp.RCC.cr().read().hsedy().bit()  {}

        // dp.RCC.cr().write(|w| w
        //     .csson().set_bit()
        // );
    }

    // GPIO_Init
    {
        dp.RCC.apb2enr().write(|w| w
            .iopden().set_bit()
            .iopaen().set_bit()
            .iopben().set_bit()
        );
        dp.GPIOB.crl().write(|w| unsafe { w
            .mode4().bits(0b01)
            .cnf4().bits(0b00)
        });
        dp.GPIOB.odr().write(|w| w
            .odr4().clear_bit()
        );
    }

    unsafe {
        cp.SYST.rvr.write(4_000_000);
        cp.SYST.cvr.write(4_000_000);
        cp.SYST.csr.write(0b101);
    }

    loop {
        if cp.SYST.has_wrapped() {
            dp.GPIOB.odr().modify(|r, w| w
                .odr4().bit(!r.odr4().bit())
            );
        }
    }
}