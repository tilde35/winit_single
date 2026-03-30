use winit_single::SingleWindow;

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
                    _ => {
                        println!("Event: {:?}", event);
                    }
                },
                winit::event::Event::AboutToWait | winit::event::Event::NewEvents(..) => {
                    // Noisy events
                }
                _ => {
                    println!("Event: {:?}", event);
                }
            }

            Ok(())
        })
    })
}
