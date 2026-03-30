# winit_single

This is a small utility library to simplify the usage of `winit` for single-window applications.

# Support Disclaimer

Little to no support will be provided for this library as I am currently occupied developing a game (Penta Terra).

Feel free to take anything from here and utilize it in your own projects or libraries. Thanks!

# Known Issues

WASM support was started, but not yet tested. There are likely bugs in that code. 

# Setup

Add the following to your `Cargo.toml` dependencies section:

```toml
[dependencies]
winit_single = { git = "https://github.com/tilde35/winit_single", branch = "winit_0_30" }
```

The `examples/simple.rs` file provides a good starting point for new applications.
