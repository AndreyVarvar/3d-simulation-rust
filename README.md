# 3D Engine
A small project of mine, where I followed [javid9x's](https://www.youtube.com/watch?v=ih20l3pJoeU) youtube tutorial on making a 3D engine.

# Requirements
In order to run this project, you need to have [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) installed.
You can check out their [README](https://github.com/Rust-SDL2/rust-sdl2/blob/master/README.md) for instructions on how to do that. I recommend using 'vcpkg' method, because it installs all dependencies at once and works on Windows, Linux and MacOS.

# Running
Just do `cargo run` to run in debug mode.
If you want maximum optimization, first do `cargo build --release`, then navigate to `./target/release/` and run `3d-simulation` executable.
