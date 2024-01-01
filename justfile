
default:
    just --list

info:
    @stat --format=%s ./target/blinky.bin

objcopyd:
    cargo size --bin blinky
    cargo objcopy --bin blinky -- -O binary ./target/blinky.bin
    @stat --format=%s ./target/blinky.bin

objcopyr:
    cargo build --release
    cargo size --bin blinky --release
    cargo objcopy --bin blinky --release -- -O binary ./target/blinky.bin
    @stat --format=%s ./target/blinky.bin

asmd:
    cargo rustc -- --emit asm

asmr:
    cargo rustc --release -- --emit asm

openocd:
    openocd -f ./stm32f103c8t6.cfg

install: 
    st-info --probe
    st-flash write ./target/blinky.bin 0x08000000