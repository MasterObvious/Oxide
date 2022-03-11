use std::error::Error;

use winit::{
    dpi::LogicalSize,
    error::OsError,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 600.0;

struct HelloTriangleApplication {
    event_loop: EventLoop<()>,
    _window: Window,
}

impl HelloTriangleApplication {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (event_loop, window) = HelloTriangleApplication::init_window()?;

        Ok(Self {
            event_loop,
            _window: window,
        })
    }

    fn init_window() -> Result<(EventLoop<()>, Window), Box<dyn Error>> {
        let event_loop = EventLoop::new();

        let window_result = WindowBuilder::new()
            .with_title("Hello Triange Application")
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .build(&event_loop);

        let window = match window_result {
            Ok(w) => w,
            Err(e) => return Err(Box::new(e)),
        };

        Ok((event_loop, window))
    }

    pub fn run(self) {
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                *control_flow = ControlFlow::Exit
            };
        });
    }
}

fn main() {
    let triangle_app = match HelloTriangleApplication::new() {
        Err(error) => panic!("Failed to create application. Cause {}", error),
        Ok(app) => app,
    };
    triangle_app.run();
}
