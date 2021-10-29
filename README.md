# rust3ds-hello-world

`Hello World` example of my Rust implementation for the Nintendo 3DS.

# Build

Make sure you have the latest Rust nightly toolchain and activate it (by defaulting it or overriding for the rust3ds-hello-world folder)
Do `rustup component add rust-src` to download the Rust source code.

Download the `ctru-sys` repository from my github profile and put it in the same directory as this git repo (support with crates.io may be thought of in the future).
Run `cargo +nightly build -Zbuild-std=core,alloc --target armv6k-nintendo-3ds.json --release` and it should build an .elf file in the target folder. Use the 3dsxtool from the devkitPRO toolchain to build it into a .3dsx and then transfer it to your console or run it in an emulator (Quicker building methods will be implemented, it just works for now).

STILL WIP.
