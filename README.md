# `rboot`

> A bootloader for HiFive1 boards written in Rust

## Dependencies

To build the project you'll need:

- Rust 1.36 or a newer toolchain. e.g. `rustup default stable`

- `rust-std` components (pre-compiled `core` crate) for the RISC-V target. Run:

``` console
$ rustup target add riscv32imac-unknown-none-elf
```

- [RISC-V toolchain for SiFive boards](https://static.dev.sifive.com/dev-tools/riscv64-unknown-elf-gcc-8.1.0-2019.01.0-x86_64-linux-ubuntu14.tar.gz)

- Programmer software
  * HiFive1 Rev B: [Segger JLink software & documentation pack for Linux](https://www.segger.com/downloads/jlink/)
  * HiFive1: [OpenOCD from SiFive](https://static.dev.sifive.com/dev-tools/riscv-openocd-0.10.0-2019.02.0-x86_64-linux-ubuntu14.tar.gz) 

## Installing the bootloader

**NOTE**: This is the very short version that only covers building programs. For
the long version, which additionally covers flashing, running and debugging
programs, check [the embedded Rust book](https://rust-embedded.github.io/book).

1. Clone the repository.

``` console
$ git clone https://github.com/Disasm/rboot
$ cd rboot
```

2. If you have an old HiFive1 board, edit `Cargo.toml`:
replace `board-hifive1-revb` with `board-hifive1`.

3. Run the programmer software.
  * HiFive1 Rev B:
```sh
/path/to/JLinkGDBServer -device FE310 -if JTAG -speed 4000 -port 3333
```
  * HiFive1:
```sh
/path/to/openocd -f board/sifive-hifive1.cfg
```

4. Build and flash the bootloader.

``` console
$ cargo run --release --features board-hifive1-revb
```

Substitute `board-hifive1-revb` with something that fits your board.
