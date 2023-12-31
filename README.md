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

在硬件层面上，MCU 具体外设只有一个，在软件层面上，采用 **单例模式**，也就是说外设只能存在一个实例；

```rust
let dp = pac::Peripheral::take().unwrap();
// 获得 Peripheral 单例；结合所有权
```







```
Peripheral_Handle.Perpheral_Register_Name.Operation

Operation ::= "read" | "modify" | "write"
```



  







