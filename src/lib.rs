mod debug;

use crate::debug::VulkanDebugger;

use std::{error::Error, ffi::CString};

use ash::{
    vk::{self, PhysicalDeviceType, QueueFamilyProperties},
    Entry, Instance,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 600.0;

pub struct HelloTriangleApplication {
    entry: Entry,
    event_loop: EventLoop<()>,
    instance: Instance,
    debugger: Option<VulkanDebugger>,
    device: ash::Device,
    _queue: vk::Queue,
    _window: Window,
}

impl HelloTriangleApplication {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let entry = Entry::linked();
        let (event_loop, window) = Self::init_window()?;

        let instance = Self::init_vulkan(&entry, &window)?;

        let debugger = match VulkanDebugger::new(&entry, &instance) {
            Some(d) => Some(d?),
            None => None,
        };

        let physical_device = Self::pick_physical_device(&instance)
            .ok_or("Unable to find suitable physical device")?;

        let device = Self::create_logical_device(&instance, physical_device)?;

        let (index, _) = Self::find_queue_families(&instance, &physical_device).unwrap();

        let queue = unsafe { device.get_device_queue(index as u32, 0) };

        Ok(Self {
            entry,
            event_loop,
            instance,
            debugger,
            device,
            _queue: queue,
            _window: window,
        })
    }

    fn create_logical_device(
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> Result<ash::Device, vk::Result> {
        let (index, _) = Self::find_queue_families(instance, &device).unwrap();

        let mut layer_names = vec![];
        VulkanDebugger::add_necessary_layers(&mut layer_names);

        let layer_names_raw = layer_names
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let queue_device_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(index as u32)
            .queue_priorities(&[1.0])
            .build();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&[queue_device_create_info])
            .enabled_layer_names(&layer_names_raw)
            .build();

        unsafe { instance.create_device(device, &device_create_info, None) }
    }

    fn find_queue_families(
        instance: &Instance,
        device: &vk::PhysicalDevice,
    ) -> Option<(usize, QueueFamilyProperties)> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*device) };

        queue_families
            .into_iter()
            .enumerate()
            .find(|(_, qf)| qf.queue_flags.contains(vk::QueueFlags::GRAPHICS))
    }

    fn is_physical_device_suitable(instance: &Instance, device: &vk::PhysicalDevice) -> bool {
        let device_properties = unsafe { instance.get_physical_device_properties(*device) };
        // let device_features = unsafe { instance.get_physical_device_features(*device) };

        device_properties.device_type == PhysicalDeviceType::DISCRETE_GPU
    }

    fn pick_physical_device(instance: &Instance) -> Option<vk::PhysicalDevice> {
        let physical_devices = unsafe { instance.enumerate_physical_devices() }.unwrap();
        physical_devices.into_iter().find(|d| {
            Self::is_physical_device_suitable(instance, d)
                && Self::find_queue_families(instance, d).is_some()
        })
    }

    fn init_vulkan(entry: &Entry, window: &Window) -> Result<Instance, vk::Result> {
        let app_info = vk::ApplicationInfo::builder()
            .application_name(CString::new("Hello Triangle").unwrap().as_c_str())
            .application_version(vk::make_api_version(1, 0, 0, 0))
            .engine_name(CString::new("No Engine").unwrap().as_c_str())
            .engine_version(vk::make_api_version(1, 0, 0, 0))
            .api_version(vk::make_api_version(0, 1, 3, 0))
            .build();

        let mut surface_extensions = ash_window::enumerate_required_extensions(&window).unwrap();

        VulkanDebugger::add_necessary_extensions(&mut surface_extensions);

        let extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let mut layer_names = vec![];
        VulkanDebugger::add_necessary_layers(&mut layer_names);

        let layer_names_raw = layer_names
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let instance_create_info_builder = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names_raw)
            .enabled_layer_names(&layer_names_raw);

        let debug_create_info = VulkanDebugger::get_debug_messenger_info();

        let instance_create_info = match debug_create_info {
            None => instance_create_info_builder.build(),
            Some(mut d) => instance_create_info_builder.push_next(&mut d).build(),
        };

        unsafe { entry.create_instance(&instance_create_info, None) }
    }

    fn init_window() -> Result<(EventLoop<()>, Window), Box<dyn Error>> {
        let event_loop = EventLoop::new();

        let window_result = WindowBuilder::new()
            .with_title("Hello Triange Application")
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .build(&event_loop);

        match window_result {
            Ok(w) => Ok((event_loop, w)),
            Err(e) => Err(Box::new(e)),
        }
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
        if let Some(debugger) = &self.debugger {
            debugger.clean_up(&self.entry, &self.instance)
        }
        unsafe { self.device.destroy_device(None) }
        unsafe { self.instance.destroy_instance(None) }
    }
}
