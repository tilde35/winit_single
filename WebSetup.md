# Web Setup (WASM)

This document contains the commands and source code for setting up a full-screen application inside a web page.

## Running WASM

After creating a WASM project, the following commands can be run in the project's root level folder. 

- Install wasm-pack (one time): `cargo install wasm-pack`
- Compile: `wasm-pack build --target web`
- Test: `wasm-pack test --headless --firefox`
- Run (after compiling, can be any web server):
    - Using Python: `python -m http.server`
    - Using Node.js: `npx serve` (note: `wgpu` gives an error when running via this method on my device)

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
    cfg.init_async(async |win, init| {
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
    });
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

This HTML expects the compiled Javascript to be in the default `pkg` folder.

```html
<!doctype html>
<html>
<!--
This allows the web app to be added to the home screen on iOS devices and run in full-screen mode without the browser's UI.
The end user must first click the "Share" button in Safari and then select "Add to Home Screen" to create the shortcut.
-->
<meta name="apple-mobile-web-app-capable" content="yes">

<head>
    <meta charset="utf-8" />
    <title>my_web_project</title>
    <style>
        html,
        body {
            margin: 0 !important;
            padding: 0 !important;
            /* Note: Background color may be shown on mobile devices (especially with horizontal orientation) */
            background-color: black;
        }

        #canvas {
            position: fixed;
            left: 0;
            top: 0;
            width: 100%;
            height: 100%;
        }
    </style>
    <script>
        function requestFullScreen() {
            const canvas = document.getElementById('canvas');
            if (canvas.requestFullscreen) {
                canvas.requestFullscreen();
            } else if (canvas.webkitRequestFullscreen) {
                canvas.webkitRequestFullscreen();
            }
        }
    </script>
</head>

<body>
    <canvas id="canvas" onclick="requestFullScreen()"></canvas>

    <script type="module">
        import init, { startup } from "./pkg/my_web_project.js";
        init().then(() => { startup(); });
    </script>
</body>

</html>
```
