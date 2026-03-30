use winit::{application::ApplicationHandler, event_loop::EventLoop};

/// A helper for creating a single window application.
///
/// Example Usage:
/// ```no_run
/// use winit_single::SingleWindow;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let cfg = SingleWindow {
///         title: "My App".to_string(),
///         ..Default::default()
///     };
///     cfg.init(|event_loop, win, init| {
///         // Perform graphics init, etc.
///         win.request_redraw();
///
///         init.run(move |event_loop, win, event| {
///             // Event handling
///             match &event {
///                 winit::event::Event::WindowEvent {
///                     window_id: _,
///                     event: winit::event::WindowEvent::CloseRequested,
///                 } => {
///                     event_loop.exit();
///                 }
///                 _ => {}
///             }
///             Ok(())
///         })
///     })
/// }
/// ```
#[derive(Debug)]
pub struct SingleWindow {
    pub title: String,
    pub inner_size: Option<[f32; 2]>,
    pub inner_size_physical: bool,
    pub position: Option<[f32; 2]>,
    pub position_physical: bool,
    pub resizable: bool,
    pub visible: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub icon: Option<([u32; 2], Vec<u8>)>,
    pub hide_cursor: bool,
    pub capture_cursor: bool,
}
impl SingleWindow {
    pub fn init(
        self,
        action: impl FnOnce(
            &winit::event_loop::ActiveEventLoop,
            &std::sync::Arc<winit::window::Window>,
            InitCallback<()>,
        ) -> Result<InitCallbackResult<()>, Box<dyn std::error::Error>>
        + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.init_event_type::<()>(action)
    }
    pub fn init_event_type<T: 'static>(
        self,
        action: impl FnOnce(
            &winit::event_loop::ActiveEventLoop,
            &std::sync::Arc<winit::window::Window>,
            InitCallback<T>,
        ) -> Result<InitCallbackResult<T>, Box<dyn std::error::Error>>
        + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut app = <SingleWindowApp<T>>::new(self, Box::new(action));
        let event_loop = <EventLoop<T>>::with_user_event().build()?;
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}
impl Default for SingleWindow {
    fn default() -> Self {
        Self {
            title: "Window".to_string(),
            inner_size: None,
            inner_size_physical: false,
            position: None,
            position_physical: false,
            resizable: true,
            visible: true,
            decorations: true,
            transparent: false,
            maximized: false,
            fullscreen: false,
            icon: None,
            hide_cursor: false,
            capture_cursor: false,
        }
    }
}

pub struct InitCallback<T>(std::marker::PhantomData<T>);
impl<T: 'static> InitCallback<T> {
    pub fn run<
        F: FnMut(
                &winit::event_loop::ActiveEventLoop,
                &std::sync::Arc<winit::window::Window>,
                winit::event::Event<T>,
            ) -> Result<(), Box<dyn std::error::Error>>
            + 'static,
    >(
        self,
        callback: F,
    ) -> Result<InitCallbackResult<T>, Box<dyn std::error::Error>> {
        Ok(InitCallbackResult {
            callback: Box::new(callback),
        })
    }
}

pub struct InitCallbackResult<T: 'static> {
    callback: Box<
        dyn FnMut(
            &winit::event_loop::ActiveEventLoop,
            &std::sync::Arc<winit::window::Window>,
            winit::event::Event<T>,
        ) -> Result<(), Box<dyn std::error::Error>>,
    >,
}

struct SingleWindowApp<T: 'static> {
    state: SingleWindowAppState<T>,
}
impl<T: 'static> SingleWindowApp<T> {
    fn new(
        cfg: SingleWindow,
        callback: Box<
            dyn FnOnce(
                &winit::event_loop::ActiveEventLoop,
                &std::sync::Arc<winit::window::Window>,
                InitCallback<T>,
            ) -> Result<InitCallbackResult<T>, Box<dyn std::error::Error>>,
        >,
    ) -> Self {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        console_error_panic_hook::set_once();

        Self {
            state: SingleWindowAppState::AwaitingResume(cfg, Some(callback), Vec::new()),
        }
    }

    fn process_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        evt: winit::event::Event<T>,
    ) {
        match &mut self.state {
            SingleWindowAppState::AwaitingResume(cfg, callback, pending_events) => {
                match evt {
                    winit::event::Event::Resumed => {
                        // Transition to the main event loop state and process pending events
                        let mut attr = winit::window::WindowAttributes::default()
                            .with_title(cfg.title.clone())
                            .with_resizable(cfg.resizable)
                            .with_visible(cfg.visible)
                            .with_decorations(cfg.decorations)
                            .with_transparent(cfg.transparent)
                            .with_maximized(cfg.maximized)
                            .with_fullscreen(if cfg.fullscreen {
                                Some(winit::window::Fullscreen::Borderless(None))
                            } else {
                                None
                            });
                        if let Some(inner_size) = cfg.inner_size {
                            if cfg.inner_size_physical {
                                attr.inner_size =
                                    Some(winit::dpi::Size::Physical(winit::dpi::PhysicalSize {
                                        width: inner_size[0] as u32,
                                        height: inner_size[1] as u32,
                                    }));
                            } else {
                                attr.inner_size =
                                    Some(winit::dpi::Size::Logical(winit::dpi::LogicalSize {
                                        width: inner_size[0] as f64,
                                        height: inner_size[1] as f64,
                                    }));
                            }
                        }
                        if let Some(position) = cfg.position {
                            if cfg.position_physical {
                                attr.position = Some(winit::dpi::Position::Physical(
                                    winit::dpi::PhysicalPosition {
                                        x: position[0] as i32,
                                        y: position[1] as i32,
                                    },
                                ));
                            } else {
                                attr.position = Some(winit::dpi::Position::Logical(
                                    winit::dpi::LogicalPosition {
                                        x: position[0] as f64,
                                        y: position[1] as f64,
                                    },
                                ));
                            }
                        }

                        #[cfg(target_arch = "wasm32")]
                        {
                            use wasm_bindgen::JsCast;
                            use winit::platform::web::WindowAttributesExtWebSys;
                            let canvas = web_sys::window()
                                .unwrap()
                                .document()
                                .unwrap()
                                .get_element_by_id("canvas")
                                .unwrap()
                                .dyn_into::<web_sys::HtmlCanvasElement>()
                                .unwrap();
                            attr = attr.with_canvas(Some(canvas));
                        }

                        let window = match event_loop.create_window(attr) {
                            Ok(w) => std::sync::Arc::new(w),
                            Err(err) => panic!("Failed to create window: {}", err),
                        };

                        if cfg.capture_cursor {
                            if let Err(err) =
                                window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                            {
                                eprintln!("Failed to capture cursor: {}", err);
                            }
                        }
                        if cfg.hide_cursor {
                            window.set_cursor_visible(false);
                        }

                        let result = if let Some(callback) = callback.take() {
                            callback(event_loop, &window, InitCallback(std::marker::PhantomData))
                                .expect("Failed to initialize window")
                        } else {
                            unreachable!(
                                "Callback should be present when transitioning from AwaitingResume to MainLoop"
                            )
                        };

                        let pending_events = std::mem::take(pending_events);

                        self.state = SingleWindowAppState::MainLoop(window, result.callback);
                        for evt in pending_events {
                            self.process_event(event_loop, evt);
                        }
                    }
                    _ => {
                        // Not a resume event, store it for later processing
                        pending_events.push(evt);
                    }
                }
            }
            SingleWindowAppState::MainLoop(window, callback) => {
                callback(event_loop, window, evt).expect("Failed to process event");
            }
        }
    }
}
impl<T: 'static> ApplicationHandler<T> for SingleWindowApp<T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let evt = winit::event::Event::Resumed;
        self.process_event(event_loop, evt);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let evt = winit::event::Event::WindowEvent { window_id, event };
        self.process_event(event_loop, evt);
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        let evt = winit::event::Event::NewEvents(cause);
        self.process_event(event_loop, evt);
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: T) {
        let evt = winit::event::Event::UserEvent(event);
        self.process_event(event_loop, evt);
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let evt = winit::event::Event::DeviceEvent { device_id, event };
        self.process_event(event_loop, evt);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let evt = winit::event::Event::AboutToWait;
        self.process_event(event_loop, evt);
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let evt = winit::event::Event::Suspended;
        self.process_event(event_loop, evt);
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let evt = winit::event::Event::LoopExiting;
        self.process_event(event_loop, evt);
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let evt = winit::event::Event::MemoryWarning;
        self.process_event(event_loop, evt);
    }
}

enum SingleWindowAppState<T: 'static> {
    AwaitingResume(
        SingleWindow,
        Option<
            Box<
                dyn FnOnce(
                    &winit::event_loop::ActiveEventLoop,
                    &std::sync::Arc<winit::window::Window>,
                    InitCallback<T>,
                )
                    -> Result<InitCallbackResult<T>, Box<dyn std::error::Error>>,
            >,
        >,
        Vec<winit::event::Event<T>>,
    ),
    MainLoop(
        std::sync::Arc<winit::window::Window>,
        Box<
            dyn FnMut(
                &winit::event_loop::ActiveEventLoop,
                &std::sync::Arc<winit::window::Window>,
                winit::event::Event<T>,
            ) -> Result<(), Box<dyn std::error::Error>>,
        >,
    ),
}
