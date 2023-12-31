
#![no_std]
#![no_main]

use panic_halt as _;

use stm32f103 as pac;

#[cortex_m_rt::entry]
fn main() -> ! {
    let _dp = pac::Peripherals::take().unwrap();
    // dp.GPIOA.odr().write(|w| unsafe { w.bits(1) } );
    loop {}
}