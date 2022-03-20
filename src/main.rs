use std::{
    error::Error,
    ffi::{c_void, CStr, CString},
};

use ash::{
    extensions::ext::DebugUtils,
    vk::{
        self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerCreateInfoEXT,
        DebugUtilsMessengerEXT, PhysicalDeviceType, QueueFamilyProperties,
    },
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

const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

const SHOULD_INCLUDE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);

struct HelloTriangleApplication {
    _entry: Entry,
    event_loop: EventLoop<()>,
    instance: Instance,
    debug_utils_messenger: Option<DebugUtilsMessengerEXT>,
    device: ash::Device,
    _queue: vk::Queue,
    _window: Window,
}

impl HelloTriangleApplication {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let entry = Entry::linked();
        let (event_loop, window) = Self::init_window()?;

        let instance = Self::init_vulkan(&entry, &window)?;

        let debug_utils_messenger = if SHOULD_INCLUDE_VALIDATION_LAYERS {
            Some(Self::init_debug_messenger(&entry, &instance))
        } else {
            None
        };
        let physical_device = Self::pick_physical_device(&instance)
            .ok_or("Unable to find suitable physical device")?;

        let device = Self::create_logical_device(&instance, physical_device)?;

        let (index, _) = Self::find_queue_families(&instance, &physical_device).unwrap();

        let queue = unsafe { device.get_device_queue(index as u32, 0) };

        Ok(Self {
            _entry: entry,
            event_loop,
            instance,
            debug_utils_messenger,
            device,
            _queue: queue,
            _window: window,
        })
    }

    unsafe extern "system" fn vulkan_debug_callback(
        _message_severity: DebugUtilsMessageSeverityFlagsEXT,
        _message_types: DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
        _p_data: *mut c_void,
    ) -> vk::Bool32 {
        let message_pointer = (*p_callback_data).p_message;
        let message = CStr::from_ptr(message_pointer);
        println!("Validation layer: {:?}", message);

        vk::FALSE
    }

    fn create_logical_device(
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> Result<ash::Device, vk::Result> {
        let (index, _) = Self::find_queue_families(instance, &device).unwrap();

        let layer_names = match SHOULD_INCLUDE_VALIDATION_LAYERS {
            true => VALIDATION_LAYERS
                .iter()
                .filter_map(|string| CString::new(*string).ok())
                .collect::<Vec<_>>(),
            false => vec![],
        };

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

    fn create_debug_messenger_create_info() -> DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(Self::vulkan_debug_callback))
            .build()
    }

    fn init_debug_messenger(entry: &Entry, instance: &Instance) -> DebugUtilsMessengerEXT {
        let debug_messenger_info = Self::create_debug_messenger_create_info();

        let debug_utils_loader = DebugUtils::new(entry, instance);
        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_messenger_info, None) }
            .unwrap()
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

        if SHOULD_INCLUDE_VALIDATION_LAYERS {
            surface_extensions.push(ash::extensions::ext::DebugUtils::name())
        }

        let extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let layer_names = match SHOULD_INCLUDE_VALIDATION_LAYERS {
            true => VALIDATION_LAYERS
                .iter()
                .filter_map(|string| CString::new(*string).ok())
                .collect::<Vec<_>>(),
            false => vec![],
        };

        let layer_names_raw = layer_names
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let instance_create_info_builder = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names_raw)
            .enabled_layer_names(&layer_names_raw);

        let mut debug_messenger_create_info = Self::create_debug_messenger_create_info();

        let instance_create_info = match SHOULD_INCLUDE_VALIDATION_LAYERS {
            true => instance_create_info_builder
                .push_next(&mut debug_messenger_create_info)
                .build(),
            false => instance_create_info_builder.build(),
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
        if let Some(debug_utils_messenger) = self.debug_utils_messenger {
            let debug_utils_loader = DebugUtils::new(&self._entry, &self.instance);
            unsafe {
                debug_utils_loader.destroy_debug_utils_messenger(debug_utils_messenger, None)
            };
        }
        unsafe { self.device.destroy_device(None) }
        unsafe { self.instance.destroy_instance(None) }
        println!("We are closing the program!")
    }
}

fn main() {
    let mut triangle_app = match HelloTriangleApplication::new() {
        Err(error) => panic!("Failed to create application. Cause: '{}'", error),
        Ok(app) => app,
    };
    triangle_app.run();
}
