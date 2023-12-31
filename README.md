```shell
$ cargo install svd2rust
$ svd2rust -i ./STM32F103.svd --target=cortex-m
$ ls
build.rs  device.x  lib.rs  STM32F103.svd
```

**form**: A library for splitting apart a large file with multiple modules into the idiomatic rust directory structure, intended for use with svd2rust. Creates a lib.rs as well as a subdirectory structure in the target directory. It does NOT create the cargo project or the cargo manifest file.

It's advised (but not necessary) to use rustfmt afterwards.

```shell
$ form -i lib.rs -o src/ && rm lib.rs
```

The resulting crate must provide an opt-in `rt` feature and depend on these crates:

-   [`critical-section`](https://crates.io/crates/critical-section) v1.x
-   [`cortex-m`](https://crates.io/crates/cortex-m) >=v0.7.6
-   [`cortex-m-rt`](https://crates.io/crates/cortex-m-rt) >=v0.6.13
-   [`vcell`](https://crates.io/crates/vcell) >=v0.1.2

Furthermore, the “device” feature of `cortex-m-rt` must be enabled when the `rt` feature is enabled. The `Cargo.toml` of the device crate will look like this:

```toml
[dependencies]
critical-section = { version = "1.0", optional = true }
cortex-m = "0.7.6"
cortex-m-rt = { version = "0.6.13", optional = true }
vcell = "0.1.2"

[features]
rt = ["cortex-m-rt/device"]
```



```toml
[package]
name = "stm32401_pactest"
version = "0.1.0"
edition = "2021"

[dependencies]
cortex-m = { version = "0.7.7", features = ["critical-section-single-core"] }
stm32f401_pac = { path = "../stm32f401_pac", features = ["rt", "critical-section"] }
panic-halt = "0.2.0"
cortex-m-rt = "0.7.2"
```

