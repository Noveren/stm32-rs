name := "blinky"
root  := "./target"

default:
    just --list

install: 
    st-info --probe
    st-flash write {{root}}/{{name + ".bin"}} 0x08000000

openocd:
    openocd -f ./stm32f103c8t6.cfg

set positional-arguments

# mode := -r | -d
build mode:
    cargo objcopy --bin {{name}} {{ if mode == "-r" {"--release"} else {""} }} -- -O binary {{root}}/{{name + ".bin"}}
    rust-size {{root + "/thumbv7m-none-eabi/"}}{{if mode == "-r" {"release/"} else {"debug/"} }}{{name}}

asm mode:
    cargo rustc --bin {{name}} {{ if mode == "-r" {"--release"} else {""} }} -- --emit asm