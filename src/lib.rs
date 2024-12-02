#[cfg(not(windows))]
compile_error!("This crate can only be built against Windows targets");

pub mod api;

#[allow(warnings)]
pub mod bindings;
pub mod plugin;
pub mod util;

use bindings::{
    UEVR_PluginInitializeParam, UEVR_PluginVersion, UEVR_PLUGIN_VERSION_MAJOR,
    UEVR_PLUGIN_VERSION_MINOR, UEVR_PLUGIN_VERSION_PATCH,
};

pub use rusty_uevr_macros::define_object;

pub unsafe fn uevr_plugin_required_version(version: *mut UEVR_PluginVersion) {
    (*version).major = UEVR_PLUGIN_VERSION_MAJOR as _;
    (*version).minor = UEVR_PLUGIN_VERSION_MINOR as _;
    (*version).patch = UEVR_PLUGIN_VERSION_PATCH as _;
}

pub unsafe fn uevr_plugin_initialize(param: *const UEVR_PluginInitializeParam) -> bool {
    if param.is_null() || (*param).callbacks.is_null() {
        return false;
    }

    api::API::initialize(param);

    if let Err(error) = std::panic::catch_unwind(|| {
        let plugin = plugin::_GLOBAL_PLUGIN
            .as_ref()
            .expect("No plugin has been registered");

        plugin.on_initialize();
    }) {
        if let Some(error) = error.downcast_ref::<&str>() {
            error!("Plugin initialization failed: {error}");
        }

        return false;
    }

    plugin::setup_callbacks((*param).callbacks, (*(*param).sdk).callbacks);

    true
}

#[macro_export]
macro_rules! define_plugin {
    ($plugin:expr) => {
        #[no_mangle]
        unsafe extern "system" fn uevr_plugin_required_version(
            version: *mut $crate::bindings::UEVR_PluginVersion,
        ) {
            $crate::uevr_plugin_required_version(version);
        }

        #[no_mangle]
        unsafe extern "system" fn uevr_plugin_initialize(
            param: *const $crate::bindings::UEVR_PluginInitializeParam,
        ) -> bool {
            $crate::uevr_plugin_initialize(param)
        }

        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "system" fn DllMain(
            _dll_module: *mut std::ffi::c_void,
            call_reason: u32,
            _reserved: *mut std::ffi::c_void,
        ) -> bool {
            if call_reason == 1 {
                let plugin = $plugin;
                plugin.on_dllmain();
                $crate::plugin::_GLOBAL_PLUGIN = Some(Box::new(plugin));
            }

            true
        }
    };
}
