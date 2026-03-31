use winit::{application::ApplicationHandler, event_loop::EventLoop};

// Re-export winit dependency
pub use winit;

pub mod prelude {
    pub use crate::{EventLoopProxy, SingleWindow};
}

/// A helper for creating a single window application.
///
/// The window can be initialized synchronously (`init`/`init_event_type`) or asynchronously (`init_async`/`init_event_type_async`).
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
    /// Synchronous initialization of the application.
    pub fn init(
        self,
        action: impl FnOnce(
            &winit::event_loop::ActiveEventLoop,
            &std::sync::Arc<winit::window::Window>,
            InitCallback<()>,
        ) -> Result<InitCallbackResult<()>, Box<dyn std::error::Error>>
        + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = <EventLoop<AppEvent<()>>>::with_user_event().build()?;
        let proxy = EventLoopProxy {
            proxy: event_loop.create_proxy(),
        };
        let mut app = <SingleWindowApp<()>>::new(
            self,
            |event_loop, win, _proxy, init| Some(action(event_loop, win, init)),
            proxy,
        );
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    /// Synchronous initialization of the application with user event support.
    ///
    /// User events can be sent using the provided `EventLoopProxy`. This allows for communication with the event loop from other
    /// threads or async contexts. This can be handled via the `winit::event::Event::UserEvent` variant in the event callback.
    pub fn init_event_type<T: 'static>(
        self,
        action: impl FnOnce(
            &winit::event_loop::ActiveEventLoop,
            &std::sync::Arc<winit::window::Window>,
            EventLoopProxy<T>,
            InitCallback<T>,
        ) -> Result<InitCallbackResult<T>, Box<dyn std::error::Error>>
        + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = <EventLoop<AppEvent<T>>>::with_user_event().build()?;
        let proxy = EventLoopProxy {
            proxy: event_loop.create_proxy(),
        };
        let mut app = <SingleWindowApp<T>>::new(
            self,
            |event_loop, win, proxy, init| Some(action(event_loop, win, proxy, init)),
            proxy,
        );
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    /// Asynchronous initialization variant of `init`.
    ///
    /// Note: The return behavior of this method is platform dependent. It may return immediately (WASM), after window is closed,
    /// or never return at all.
    pub fn init_async<
        F: std::future::Future<Output = Result<InitCallbackResult<()>, Box<dyn std::error::Error>>>
            + 'static,
    >(
        self,
        action: impl FnOnce(std::sync::Arc<winit::window::Window>, InitCallback<()>) -> F + 'static,
    ) {
        self.init_event_type_async::<(), _>(move |win, _proxy, init| action(win, init))
    }

    /// Asynchronous initialization variant of `init_event_type`.
    ///
    /// User events can be sent using the provided `EventLoopProxy`. This allows for communication with the event loop from other
    /// threads or async contexts. This can be handled via the `winit::event::Event::UserEvent` variant in the event callback.
    ///
    /// Note: The return behavior of this method is platform dependent. It may return immediately (WASM), after window is closed,
    /// or never return at all.
    pub fn init_event_type_async<
        T: 'static,
        F: std::future::Future<Output = Result<InitCallbackResult<T>, Box<dyn std::error::Error>>>
            + 'static,
    >(
        self,
        action: impl FnOnce(
            std::sync::Arc<winit::window::Window>,
            EventLoopProxy<T>,
            InitCallback<T>,
        ) -> F
        + 'static,
    ) {
        let event_loop = <EventLoop<AppEvent<T>>>::with_user_event()
            .build()
            .expect("Failed to build event loop");
        let proxy = EventLoopProxy {
            proxy: event_loop.create_proxy(),
        };
        let mut app = <SingleWindowApp<T>>::new(
            self,
            |_event_loop, win, proxy, init| {
                // Web platforms: Spawn the async initialization without blocking
                #[cfg(target_arch = "wasm32")]
                {
                    let win = win.clone();
                    let proxy = proxy.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let result = action(win, proxy.clone(), init).await;
                        proxy
                            .proxy
                            .send_event(AppEvent::DeferredInit(result))
                            .ok()
                            .expect("Failed to send deferred init event");
                    });
                    return None;
                }

                // Native platforms: Block on the async initialization once the window is ready.
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let result = pollster::block_on(action(win.clone(), proxy, init));
                    return Some(result);
                }
            },
            proxy,
        );
        event_loop.run_app(&mut app).expect("Application error");
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

pub struct EventLoopProxy<T: 'static> {
    proxy: winit::event_loop::EventLoopProxy<AppEvent<T>>,
}
impl<T: 'static> EventLoopProxy<T> {
    pub fn send_user_event(
        &self,
        event: T,
    ) -> Result<(), winit::event_loop::EventLoopClosed<AppEvent<T>>> {
        self.proxy.send_event(AppEvent::UserEvent(event))
    }
}
impl<T: 'static> Clone for EventLoopProxy<T> {
    fn clone(&self) -> Self {
        Self {
            proxy: self.proxy.clone(),
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
        callback: impl FnOnce(
            &winit::event_loop::ActiveEventLoop,
            &std::sync::Arc<winit::window::Window>,
            EventLoopProxy<T>,
            InitCallback<T>,
        )
            -> Option<Result<InitCallbackResult<T>, Box<dyn std::error::Error>>>
        + 'static,
        proxy: EventLoopProxy<T>,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        Self {
            state: SingleWindowAppState::AwaitingResume(
                cfg,
                Some((Box::new(callback), proxy)),
                Vec::new(),
            ),
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

                        let result = if let Some((callback, proxy)) = callback.take() {
                            callback(
                                event_loop,
                                &window,
                                proxy,
                                InitCallback(std::marker::PhantomData),
                            )
                        } else {
                            unreachable!(
                                "Callback should be present when transitioning from AwaitingResume to MainLoop"
                            )
                        };

                        let pending_events = std::mem::take(pending_events);

                        match result {
                            Some(result) => {
                                let callback = result.expect("Initialization failed").callback;
                                self.state = SingleWindowAppState::MainLoop(window, callback);
                                for evt in pending_events {
                                    self.process_event(event_loop, evt);
                                }
                            }
                            None => {
                                self.state =
                                    SingleWindowAppState::AwaitingInit(window, pending_events);
                            }
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
            SingleWindowAppState::AwaitingInit(_window, events) => {
                // Still waiting for initialization to complete, store events for later processing
                events.push(evt);
            }
            SingleWindowAppState::Placeholder => {
                unreachable!("Placeholder state should never be active");
            }
        }
    }
}
impl<T: 'static> ApplicationHandler<AppEvent<T>> for SingleWindowApp<T> {
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

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: AppEvent<T>) {
        match event {
            AppEvent::UserEvent(u) => {
                let evt = winit::event::Event::UserEvent(u);
                self.process_event(event_loop, evt);
            }
            AppEvent::DeferredInit(result) => {
                let callback = result.expect("Deferred initialization failed").callback;
                let state = std::mem::replace(&mut self.state, SingleWindowAppState::Placeholder);
                if let SingleWindowAppState::AwaitingInit(window, pending_events) = state {
                    self.state = SingleWindowAppState::MainLoop(window, callback);
                    for evt in pending_events {
                        self.process_event(event_loop, evt);
                    }
                } else {
                    panic!("DeferredInit event received in invalid state");
                }
            }
        }
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

pub enum AppEvent<T: 'static> {
    UserEvent(T),
    DeferredInit(Result<InitCallbackResult<T>, Box<dyn std::error::Error>>),
}

enum SingleWindowAppState<T: 'static> {
    AwaitingResume(
        SingleWindow,
        Option<(
            Box<
                dyn FnOnce(
                    &winit::event_loop::ActiveEventLoop,
                    &std::sync::Arc<winit::window::Window>,
                    EventLoopProxy<T>,
                    InitCallback<T>,
                )
                    -> Option<Result<InitCallbackResult<T>, Box<dyn std::error::Error>>>,
            >,
            EventLoopProxy<T>,
        )>,
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
    AwaitingInit(
        std::sync::Arc<winit::window::Window>,
        Vec<winit::event::Event<T>>,
    ),
    Placeholder,
}
