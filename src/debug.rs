use std::ffi::{c_void, CStr, CString};

use ash::{
    extensions::ext::DebugUtils,
    vk::{
        self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerCreateInfoEXT,
        DebugUtilsMessengerEXT,
    },
    Entry, Instance,
};

const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

const SHOULD_INCLUDE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);
pub struct VulkanDebugger {
    debug_utils_messenger: DebugUtilsMessengerEXT,
}

impl VulkanDebugger {
    unsafe extern "system" fn vulkan_debug_callback(
        message_severity: DebugUtilsMessageSeverityFlagsEXT,
        _message_types: DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
        _p_data: *mut c_void,
    ) -> vk::Bool32 {
        let message_pointer = (*p_callback_data).p_message;
        let message = CStr::from_ptr(message_pointer);

        if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::ERROR) {
            log::error!("{}", message.to_str().unwrap());
        } else if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::WARNING) {
            log::warn!("{}", message.to_str().unwrap());
        } else if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::INFO) {
            log::info!("{}", message.to_str().unwrap());
        } else if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::VERBOSE) {
            log::debug!("{}", message.to_str().unwrap());
        }
        vk::FALSE
    }

    pub fn new(entry: &Entry, instance: &Instance) -> Option<Result<Self, vk::Result>> {
        if !SHOULD_INCLUDE_VALIDATION_LAYERS {
            return None;
        }

        let debug_utils_messenger = match Self::init_debug_messenger(entry, instance) {
            Ok(d) => d,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok(Self {
            debug_utils_messenger,
        }))
    }

    pub fn add_necessary_extensions(extension_list: &mut Vec<&CStr>) {
        if SHOULD_INCLUDE_VALIDATION_LAYERS {
            extension_list.push(DebugUtils::name());
        }
    }

    pub fn add_necessary_layers(layer_list: &mut Vec<CString>) {
        if SHOULD_INCLUDE_VALIDATION_LAYERS {
            layer_list.extend(
                VALIDATION_LAYERS
                    .iter()
                    .filter_map(|string| CString::new(*string).ok()),
            );
        }
    }

    pub fn get_debug_messenger_info() -> Option<DebugUtilsMessengerCreateInfoEXT> {
        SHOULD_INCLUDE_VALIDATION_LAYERS.then(Self::create_debug_messenger_create_info)
    }

    fn init_debug_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> Result<DebugUtilsMessengerEXT, vk::Result> {
        let debug_messenger_info = Self::create_debug_messenger_create_info();

        let debug_utils_loader = DebugUtils::new(entry, instance);
        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_messenger_info, None) }
    }

    fn create_debug_messenger_create_info() -> DebugUtilsMessengerCreateInfoEXT {
        DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                DebugUtilsMessageSeverityFlagsEXT::INFO
                    | DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(Self::vulkan_debug_callback))
            .build()
    }

    pub fn clean_up(&self, entry: &Entry, instance: &Instance) {
        let debug_utils_loader = DebugUtils::new(entry, instance);
        unsafe {
            debug_utils_loader.destroy_debug_utils_messenger(self.debug_utils_messenger, None)
        };
    }
}
