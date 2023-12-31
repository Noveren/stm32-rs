在 Embedded Rust 中，MCU 最顶层的抽象是 Peripheral Access Crate，简称 PAC；PAC 提供了访问控制器寄存器去配置和控制 MCU 的功能；在 PAC 之上是 Hardware Abstraction Layer，简称 HAL，提供更高层级的抽象和安全代码保证；虽然一些教程使用 HAL 进行演示，但是在代码中总是会混合其他层级，如 PAC；在 Rust 中很容易混合多种抽象层级，但是这在学习中是很令人困惑，使学习者难以确定自己是否真的理解了这些抽象概念。

## 0x00 Rust Cortex-M 开发环境

```
Software ======================================== Crate
    High                   Board
     |    ---------------------------------------
     |          Hardware Abstraction Layer
     |    ---------------------------------------
    Low   Micro-Architecture | Peripheral Access
Hardware ========================================
          Microprocessor     | Peripheral
```

Rust 的嵌入式生态中提供了不同层级的 Crate 封装。Cortex-M 团队针对 ARM 的 Cortex-M 微处理器主要提供了 `cortex-m` 和 `cortex-m-rt` 两个 Crate

+ **`cortex-m`**：Cortex-M 微处理器低层级封装，如系统时钟、核心外设、核心寄存器等等

+ **`cortex-m-rt`**：启动代码 Startup Code 和 Cortex-M 最小化运行时

## 0x01 Peripheral Access Crate

svd2rust 是 Embedded Rust 生态中的用于将 CMSIS System View Description [CMSIS-SVD](https://www.keil.com/pack/doc/CMSIS/SVD/html/index.html) 文件转换为 Rust Crate 的命令行工具；SVD 文件采用 XML 格式，描述了 MCU 的硬件特性，列出了 MCU 所有可用外设，包括寄存器在内存中的位置以及寄存器对应的功能

```shell
$ cargo install svd2rust
$ cargo install form
```

svd2rust 支持 `cortex-m`、`msp430`、`riscv`、`xtensa-lx`，当不指定 `--target=` 时，默认以 `cortex-m` 为生成目标

```shell
$ svd2rust -i ./STM32F103.svd --target=cortex-m
$ ls
build.rs  device.x  lib.rs
$ cargo init device --lib
$ rm src/lib.rs
$ form -i lib.rs -o src && rm lib.rs    # 分割原始 lib.rs
```

```shell
# svd2rust 可选项
--reexport-core-peripherals     cortex-m;
--reexport-interrupt            cortex-m-rt;
```

查阅 svd2rust 的 [文档](https://docs.rs/svd2rust/latest/svd2rust/)，完成代码生成后还需要配置 crate 的依赖；参考配置如下：

```toml
[dependencies]
critical-section = { version = "1.0", optional = true }
cortex-m = "0.7.6"
cortex-m-rt = { version = "0.7.1", optional = true }
vcell = "0.1.2"

[features]
rt = ["cortex-m-rt/device"]
```

至此完成基于某型号 MCU 的 SVD 文件生成 PAC crate，后续就是使用 PAC 进行开发

## 0x02 基础模板

```shell
$ cargo new --bin demo && cd demo
```

然后配置基础依赖，主要是内核、启动代码及运行时、PAC，具体版本需要自行查阅文档

```toml
[dependencies]
cortex-m-rt = "0.7.1"

[dependencies.cortex-m]
version = "0.7.6"
features = ["critical-section-single-core"] # 注意开启；缺少将导致链接失败

[dependencies.stm32f103]
path = "../stm32f103"
features = ["rt", "critical-section"] 	    # 注意开启；第二个缺少将导致没有 take 方法
```

```toml
# .cargo/config.toml
[target.thumbv7m-none-eabi]
rustflags = [
  "-C", "link-arg=-Tlink.x",
]

[build]
target = "thumbv7m-none-eabi"
```

在 `main.rs` 中，需要为 `cortex-m-rt` 提供程序入口，以及为 Rust 提供 `panic_handler`

```rust

#![no_std]           // core 环境
#![no_main]          // 无 main 函数


// Panic: panic-halt = "0.2.0" 内部代码
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
// Panic: 用 `use panic_halt as _;` 可替代以上代码

use stm32f103 as pac;

#[cortex_m_rt::entry]
fn main() -> ! {
    loop {}
}
```

## 0x03 PAC 使用模式

```rust
struct Peripherals {
    // pub PPP: PPP,          // 各种外设成员
}
```

```rust
impl Peripherals {
    #[cfg(feature = "critical-section")]
    pub fn take() -> Option<Self>;
    
    pub unsafe fn steal() -> Self;
}
```

```rust
let dp = pac::Peripheral::take();   // 一般不需要 mut（取决于内部实现）
```

类型 `Peripherals` 为单例类型，可以通过 `take` 类型函数 **安全单例化**，采取的方法为检查 `static mut DEVICE_PERIPHERALS: bool = false;` 全局变量标志，内部调用 **不安全实例化** 的 `steal` 类型函数（内部置位 `DEVICE_PERIPHERALS`）

`Peripherals` 单例以 **所有非内核外设的寄存器** 为成员，并作为访问这些寄存器的句柄 Handle 被使用，模式如下为 `peripherals.PPP.register.operation`

**寄存器获取**：所有成员 PPP 的具体外设名称、具体 register 名称、具体 register 某位名称都在 SVD 文件中定义，一般与相应 MCU 参考手册中命名一致，且都采用 **`const fn` 的函数模式**，进行零成本的类型转换

**寄存器访问**：对于 `.register()` 返回的寄存器类型，基本的操作方式如下

```rust
fn reset(&self)              // 重置寄存器的所有位
fn read(&self) -> R<REG>     // 继续通过链式调用选择寄存器的位
```

```rust
fn write(&self, f: F)        // 写
where
	F: FnOnce<&mut W<REG>) -> &mut W<REG>

// 调用时传入匿名参数 |w| ...
// w 通过 不断链式调用 选择寄存器的位、修改方法
// 调用一次修改方法后，又可以继续选择寄存器的其他位
```

```rust
fn modeify<F>(&self, f: F)    // 修改：读并写
where
	for<'w> F: FnOnce(&R<REG>, &'w mut W<REG>) -> &'w mut W<REG>

// 调用时传入匿名函数 |r, w| ...
// r 表示寄存器，可以用于读取某些位
```

**注意**：寄存器会按照 `read-only`、``write-only`、`read-write` 暴露部分访问接口

**中断枚举**：SVD 文件描述了 MCU 的中断，svd2rust 生成的 PAC 中导出了 MCU 中断的枚举 `Interrupt`，可以配合 `cortex-m` 进行使用：

```rust
use cortex_m::peripheral::Peripherals;
use stm32f30x::Interrupt;

let p = Peripherals::take().unwrap();
let mut nvic = p.NVIC;

nvic.enable(Interrupt::TIM2);
nvic.enable(Interrupt::TIM3);
```

**`rt`** 特性：当该特性允许时，PAC 将会启用 `cortex-m-rt` 的 `device` 特性，即提供 `device.x` 用于生成链接脚本；另外，如果使用 svd2rust 进行转换时在命令行传入 `--reexport-interrupt` 选项，那么 PAC 还会提供 `interrupt!`（非 Cortex-M 及 MSP430 的 MCU）或 `#[interrupt]` 用于注册中断服务函数

## 0x04 Cortex-m 内核

### 1. cortex-m

**`cortex-m`**：Cortex-M 处理器的低层级访问接口，如内核外设访问、内核寄存器访问、中断操作方法、Cortex-M 特殊指令的安全封装；另外还有其他需要通过条件编译打开的特性：

+ `inline-asm`：当该特性允许时，所有在 `asm` 和 `register` 模块中的实现都将使用内联汇编宏 `asm!`，而不是目前使用的外部汇编器；内联汇编宏的好处一是减少开销，二是一些 `register` 中的 API 只能在内联汇编实现时可用；缺陷是要求 Rust 的版本在 1.59 以上；**在未来的 0.8 及更高版本中，这个特性将总是允许**
+ `critical-section-single-core`：该特性基于失能全局中断，适用于 **单核** 目标的 `critical-section` crate 的一些实现；注意不要在多核目标或非特权代码中使用
+ `cm7-r0p1`：Cortex-M7 相关功能支持
+ `linker-plugin-lto`：高级链接特性支持

该 crate 同样通过 `Peripherals` 的方式，类似于 PAC，提供内核外设的访问；

另外，在构建时，该 crate 的 `build.rs` 将检测编译 Cargo 配置的 `target` 的以 `thumb` 开头的值，如 `thumbv7m-none-eabi`，并按照具体内容向 `rust-cfg` 提供参数

### 2. cortex-m-rt

**`cortex-m-rt`**：Cortex-M MCU的 `startup code` 和 `minimal runtime`；这个 crate 包含构建 Cortex-M MCU `no_std` 应用所需的所有部分：

**链接脚本**：定义程序在存储器中的布局，特别是填充内存空间中规定位置的中断向量表，从而设备可以正确的启动，以及分派 `exception` （处理器）和 `interrupt`（微控制器）

+ **`memory.x`**：FLASH 和 RAM 都不属于内核外设，因此需要用户提供（一般在项目根目录）下提供 `memory.x` 文件，并在其中定义 FLASH 和 RAM 的起始地址和长度
    另外 `memory.x` 中还可以可选的提供 `_stack_start` 和 `_stext` 符号

```ld
MEMORY
{
	FLASH : ORIGIN = 0x08000000, LENGTH = 64K
	RAM : ORIGIN = 0x20000000, LENGTH = 20K
}
```

+ **`device.x`**：当 `device` 特性启用时，用户需要通过 `device.x` 提供中断向量表的填充地址，该文件在使用 `svd2rust` 时将会自动生成；当 `device` 特性未启用时，或中断向量表未完全填充且 `svd2rust` 生成库启用 `rt` 时，`cortex-m-rt` 将自动填充默认值
    中断向量表是指定过程或函数的地址；在 C 项目中，一般规定中断函数的名称，并采用 weak 定义为默认 Handler；在 `cortex-m-rt` 中，通过 `device.x` 的 `PROVICDE` 功能实现 C 项目中类似的效果；
    虽然 `cortex-m-rt` 提供了 `#[interrupt]` 属性宏用于对用户中断函数进行注释，但是由于不同微控制器拥有不同狭义中断（非异常），因此需要通过 PAC **重导出** 后使用
    另外为了使用 `device.x`，PAC 库的 `build.rs` 还得做一些处理，但是总之，PAC 的相关任务交给 `svd2rust` 即可
+ **`link.x`**：`cortex-m-rt` 提供了 `link.x.in` 链接脚本模版，编译时将会整合 `memory.x` 和 `device.x` 以及代码中的相关内容，生成 `link.x` 送入链接器，因此在配置 Cargo 时需要提供

```toml
[target.thumbv7m-none-eabi]
rustflags = [
  "-C", "link-arg=-Tlink.x",
]
```

**启动代码**：在 **进入 entry point 前**，在 RAM 中初始化静态变量，以及启用特有功能

+ **`#[entry]`**：将一个函数指定为应用入口

```rust
#[entry]
fn main() -> ! {
    loop { }
}
```

+ **`#[expection]`**：注释异常（内核外设或机制产生的中断）函数；异常的 Handler 默认是无限循环
+ **`#[pre_init]`**：在初始化静态变量之前执行的函数，类似于 C 中的 `SystemInit`

## 0x05 封装与抽象

一方面，svd2rust 生成的 PAC 库只提供了寄存器级的操作，用户进行配置时需要频繁查阅手册；另一方面，由于 Rust 将安全性融入到了语法中，像 C 项目中常用的代码风格在 Rust 中通常是 Unsafe 的，如全局变量，为了达成某种效果，可能需要非常繁琐的写法，并且对用户的 Rust 水平有较高要求

**`embedded hal`** 是一个由 HAL team 维护的 crate，其利用 Rust 的 Trait 机制，定义了非常强大的概念抽象；一款 MCU 的 HAL crate 可以通过实现 `embedded hal` 来设计，若业务代码只基于这样的 HAL 开发，那么这份业务代码就很容易在不同 MCU 之间移植；同时，相同的概念抽象能够约束具体 MCU 的 HAL 的设计，降低用户的学习成本

```shell
stm32f1xx-hal <- 实现 - embedded_hal
              <- 依赖 - PAC, ......
```

`stm32f1xx-hal` 是一个实现了 `embedded_hal` 的适用于 STM32F1xx 的 HAL；基于 HAL 开发和基于 PAC 开发有所不同，以下仅简单介绍，具体使用教程可查阅 [github 仓库](https://github.com/stm32-rs/stm32f1xx-hal)

```rust
let dp = pac::Peripherals::take().unwrap();

// 在 stm32f1xx-hal 中为 pac 中的 PPP 类型实现 Trait - PPPExt
// 因此相对于原始 PAC，从 stm32f1xx-hal 中导出的 PAC 的 pac::PPP 有更多方法
let mut falsh = dp.FLASH.constrain();
let mut rcc   = dp.RCC.constrain();
let mut afio  = dp.AFIO.constrain();

// 获取、访问方式与 PAC 库有所不同
let clocks = rcc.cfgr.freeze(&mut flash.acr);
```

项目若要使用 `stm32f1xx-hal`，Cargo.toml 配置方式如下

```toml
[dependencies]
cortex-m     = "0.7.6"
cortex-m-rt  = "0.7.1"
panic-halt   = "0.2.0"				# 可选
embedded-hal = "0.2.7"              # 可选
nb           = "1"                  # 可选

[dependencies.stm32f1xx-hal]
version = "0.10.0"
# rt -> stm32f1/rt -> cortex-m-rt/device
# stm32f103 具体型号
# medium FLASH/SRAM 容量；stm32f1xxXYxx Y 指容量，Y = 8,B => medium
features = ["rt", "stm32f103", "medium"]
```

另外，介绍一下 **`nb`**：最小化和可重用的非阻塞 IO 层；这个 crate 的最终目标是 **代码复用**；基于 `nb` 用户编写的核心 IO API 可以被转换为阻塞模式或非阻塞模式，进一步的，这些 API 不局限于具体的异步模型，可以工作于 `futures` 模型或 `async/await` 模型；`stm32f1xx-hal` 的一些 API 就用到了 `nb`，如

```rust
nb::block!(timer.wait()).unwrap();            // 将 timer.wait() 转换为阻塞模式执行
```

## 0x06 烧录与调试

### 1. 概述

烧录指的是将编译好的机器代码写入到嵌入式系统的存储器中，完成配置后，这些机器代码可以被 CPU 读取并执行；烧录的方式可以分为以下三种：

+   **在电路编程 ICP**：使用嵌入式系统的烧录接口电路或机制，如 以Cortex-M3 内核的 MCU 通常支持的 SWD 或 JTAG 调试接口，配合特定的适配器进行烧录
+   **在系统编程 ISP**：通过如 USB、UART、SPI 等通讯接口，利用微控制器自身已有程序，如 BootLoader，引导代码写入存储器
+   **在应用编程 IAP**：从结构上将Flash存储器映射为两个存储体，当运行一个存储体上的用户程序时，可对另一个存储体重新编程，之后将程序从一个存储体转向另一个，如 **固件自动更新**

**ICP** 通常使用 MCU 的调试系统来实现，对于 MCU 来说，主要是片上电路支持的调试协议，片上调试接口通过调试器进行使用，PC 上又可以通过软件来使用调试器对微控制器进行调试，常用协议如下：

+   **JTAG**：全称 Joint Test Action Group，是一种国际标准测试协议（IEEE 1149.1兼容），主要用于芯片内部测试，目前高级器件，如 MCU、DSP、FPGA 等，都支持该协议
+   **SWD**：全称 Serial Wire Debug，是一种 32 位 ARM 内核调试器的一种同步调试协议，相对于 JTAG 更为简单，除了电源线和地线外，`SWDIO`、`SWCLK` 分别作为数据线和时钟线

ST-LINK 由 ST 意法半导体官方推出调试器和烧录器，用于 STM8 和 STM32 微控制器的开发，向上通过 USB 连接 PC，向下通过 SWIM/SWD/JTAG 连接微控制器。除了官方推出的 ST-LINK 设备外，还有淘宝上 ST-LINK V2 售卖的 U 盘型调试器，便宜可用

ST-LINK 可以通过 Keil IDE 使用，也可以通过开源的 [stlink](https://github.com/stlink-org/stlink) 工具集使用，还可以通过开源的 OpenOCD 使用

+   **stlink**：基于 ST-LINK 调试器的开源工具集，适用于 STM32 的调试和编程

    ```shell
    # stlink 相关工具
    $ st-info --probe								# chip information tool
    $ st-flash write <file.bin> 0x08000000			# download
    ```

+   **OpenOCD**：全称 Open On-Chip Debugger，旨在为嵌入式目标设备提供调试、系统内编程、边界扫描的功能，对于不同调试协议，需要配合相应的硬件调试器使用；OpenOCD 能够提供：烧录、GDB 服务端、Semihosting 等等功能

    ```
    # ./stm32f103c8t6.cfg
    source [find interface/stlink-v2.cfg]
    source [find target/stm32f1x.cfg]
    
    # halt target after gdb attached
    $_TARGETNAME configure -event gdb-attach { halt }
    ```

    ```shell
    $ openocd -f ./stm32f103c8t6.cfg
    ```

    OpenOCD 能够启动一个 GDB 服务器，默认端口号为 `localhost:3333`，可以通过 arm-none-eabi-gdb GDB 客户端进行连接；命令行 GDB 学习成本相对较高，在 VSCode 中可以使用 Cortex-Debug 插件进行调试，参考配置如下：

    ```json
    {
        "version": "0.2.0",
        "configurations": [
            {
                "name": "Cortex Debug",
                "cwd": "${workspaceFolder}",
                "executable": "./build/demo.elf",
                "request": "launch",
                "type": "cortex-debug",
                "runToEntryPoint": "main",
                "servertype": "external",
                "gdbTarget": "localhost:3333",
                "gdbPath": "arm-none-eabi-gdb"
            }
        ]
    }
    ```

**注**：Semihosting 是一种机制：让嵌入式设备在宿主环境中执行输入输出，即在宿主控制台上输入日志信息，便于调试。该机制依赖于硬件调试器，如 ST-Link，以及 OpenOCD

### 2. OpenOCD

```shell
$ openocd --help
Open On-Chip Debugger 0.12.0 (2023-01-14-23:37)
--help       | -h       display this help
--version    | -v       display OpenOCD version
--file       | -f       use configuration file <name>
--command    | -c       run <command>
--log_output | -l       redirect log output to file <name>
```

OpenOCD 的启动需要读取一个配置文件 `.cfg`，该文件中批量给出了 OpenOCD 的配置指令；

OpenOCD 的启动需要执行一系列配置指令，这些指令可以通过 `-c` 传递，也可以从 `.cfg` 配置文件中读取；当无参数启动时，OpenOCD 会检查并使用当前目录下名为 `openocd.cfg` 的文件，用户可以通过 `--file/-f` 指定配置文件；OpenOCD 提供了常用 `interface`、`target`、`board` 的配置文件

```shell
$ openocd -f interface/stlink.cfg -f target/stm32f1x.cfg
```

OpenOCD  在成功启动后将作为 **服务器** 运行；一旦 OpenOCD 作为一个服务器运行，它将会等待客户端（ Telnet、GDB、RPC）的连接，并且会处理来自这些客户端的命令；OpenOCD 为 GDB 默认提供的端口号为 `localhost:3333`

```shell
$ arm-none-eabi-gdb
(gdb) target extended-remote :3333
(gdb) tar ext :3333
(gdb) file <elf>
(gdb) load
(gdb) continue
(gdb) b main
```

```shell
(gdb) monitor <openocd-run-stage-command>
```

**服务端指令**

```shell
help [command-name]                     # 帮助
shutdown [error]                        # 关闭 OpenOCD Server
echo [-n] string                        # -n 表示接在上一行后
debug_level [n]                         # 改变调试等距 n ::= 0 | 1 | 2 | 3 | 4
log_output [filename | "default"]       # 日志重定向；default is stderr
```

**目标状态接口**

```shell
halt [ms]
wait_halt [ms]                          # wait for target to halt and enter debug mode
reset
reset run
reset halt
resume
```

**内存访问指令**

**镜像载入指令**

```shell
dump_image <filename> <address> <size>          # 读出设备上现有镜像
load_image                                      # 尝试失败
```

```shell
monitor arm semihosting [enable | disable]
monitor arm semihosting_redirect tcp localhost:3333      # TODO
```

Semihosting提供了一种机制，让CPU上的代码与主机通信并借助主机侧的功能。其核心原理就是，在CPU侧执行特定序列的指令，在主机侧识别这些指令，并采集参数，调用主机侧响应功能。





























