# Web Setup (WASM)

## Running on the Web

- Install wasm-pack (one time): `cargo install wasm-pack`
- Compile: `wasm-pack build --target web`
- Test: `wasm-pack test --headless --firefox`
- Run (after compiling, can be any web server): `python -m http.server`

## Web Project Setup

### src/lib.rs

```rust
use wasm_bindgen::prelude::*;
use winit_single::{SingleWindow, winit};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn startup() {
    let cfg = SingleWindow {
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
                    _ => {
                        log(&format!("Event: {:?}", event));
                    }
                },
                winit::event::Event::AboutToWait | winit::event::Event::NewEvents(..) => {
                    // Noisy events
                }
                _ => {
                    log(&format!("Event: {:?}", event));
                }
            }

            Ok(())
        })
    })
    .unwrap();
}
```

### Cargo.toml

Note: replace `my_web_project` with your project name

```toml
[package]
name = "my_web_project"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2.84"
winit_single = { git = "https://github.com/tilde35/winit_single", branch = "winit_0_30" }

[dev-dependencies]
wasm-bindgen-test = "0.3.34"

[profile.release]
# Tell `rustc` to optimize for small code size.
opt-level = "s"
```

### index.html

Note: replace `my_web_project` with your project name

This HTML expects the compiled Javascript to be in the `pkg` folder, see `script` section.

```html
<!doctype html>
<html>

<head>
    <meta charset="utf-8" />
    <title>my_web_project</title>
    <style>
        html,
        body {
            width: 100%;
            height: 100%;
            margin: 0px;
            padding: 0px;
        }
        canvas {
            display: block;
            width: 100%;
            height: 100%;
            margin: 0px;
            padding: 0px;
        }
    </style>
</head>

<body>
    <canvas id="canvas"></canvas>

    <script type="module">
        import init, { startup } from "./pkg/my_web_project.js";
        init().then(() => { startup(); });
    </script>
</body>

</html>
```
