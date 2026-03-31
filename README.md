# winit_single

This is a small utility library to simplify the usage of `winit` for single-window applications.

# Support Disclaimer

Little to no support will be provided for this library as I am currently occupied developing a game (Penta Terra).

Feel free to take anything from here and utilize it in your own projects or libraries, no credit required. Thanks!

# Setup

## Desktop Setup

Add the following to your `Cargo.toml` dependencies section:

```toml
[dependencies]
winit_single = { git = "https://github.com/tilde35/winit_single", branch = "winit_0_30" }
```

Quick starting point for `main.rs`:

```rust
use winit_single::{SingleWindow, winit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = SingleWindow {
        title: "Simple App".to_string(),
        ..Default::default()
    };
    cfg.init(|_event_loop, win, init| {
        // Perform graphics init, etc.
        win.request_redraw();

        init.run(move |event_loop, win, event| {
            match &event {
                winit::event::Event::WindowEvent {
                    window_id: _,
                    event: w,
                } => match w {
                    winit::event::WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        win.request_redraw();
                        // Perform drawing here
                    }
                    _ => {}
                },
                _ => {}
            }

            Ok(())
        })
    })
}
```

## Web Setup (WASM)

See [WebSetup.md](WebSetup.md) for details.
