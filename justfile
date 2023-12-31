
default:
    just --list

info:
    arm-none-eabi-size ./target/thumbv7m-none-eabi/release/blinky

objcopy:
    arm-none-eabi-objcopy ./target/thumbv7m-none-eabi/release/blinky ./target/blinky.bin -O binary
    arm-none-eabi-size ./target/thumbv7m-none-eabi/release/blinky
    @stat --format=%s ./target/blinky.bin

install: 
    st-info --probe
    st-flash write ./target/blinky.bin 0x08000000