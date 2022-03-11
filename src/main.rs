use std::{error::Error, ffi::CString};

use ash::{vk, Entry, Instance};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 600.0;

struct HelloTriangleApplication {
    entry: Entry,
    event_loop: EventLoop<()>,
    instance: Instance,
    window: Window,
}

impl HelloTriangleApplication {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let entry = Entry::linked();
        let (event_loop, window) = HelloTriangleApplication::init_window()?;

        let instance = HelloTriangleApplication::init_vulkan(&entry, &window)?;

        Ok(Self {
            entry,
            event_loop,
            instance,
            window,
        })
    }

    fn init_vulkan(entry: &Entry, window: &Window) -> Result<Instance, vk::Result> {
        let app_info = vk::ApplicationInfo::builder()
            .application_name(CString::new("Hello Triangle").unwrap().as_c_str())
            .application_version(vk::make_api_version(1, 0, 0, 0))
            .engine_name(CString::new("No Engine").unwrap().as_c_str())
            .engine_version(vk::make_api_version(1, 0, 0, 0))
            .api_version(vk::make_api_version(1, 0, 0, 0))
            .build();

        let surface_extensions = ash_window::enumerate_required_extensions(&window).unwrap();
        let extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names_raw)
            .build();

        unsafe { entry.create_instance(&instance_create_info, None) }
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

    pub fn run(&mut self) {
        self.event_loop.run_return(move |event, _, control_flow| {
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

impl Drop for HelloTriangleApplication {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) }
        println!("We are closing the program!")
    }
}

fn main() {
    let mut triangle_app = match HelloTriangleApplication::new() {
        Err(error) => panic!("Failed to create application. Cause {}", error),
        Ok(app) => app,
    };
    triangle_app.run();
}
