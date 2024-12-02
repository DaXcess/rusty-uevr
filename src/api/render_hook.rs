use crate::{
    bindings::{UEVR_FRenderTargetPoolHookFunctions, UEVR_IPooledRenderTargetHandle},
    util::encode_wstr,
};

use std::ptr::null;

static mut STATIC_RENDER_HOOK: *const UEVR_FRenderTargetPoolHookFunctions = null();

pub fn activate() {
    let fun = initialize().activate.unwrap();

    unsafe { fun() }
}

pub fn get_render_target(name: impl AsRef<str>) -> UEVR_IPooledRenderTargetHandle {
    let name = encode_wstr(name);
    let fun = initialize().get_render_target.unwrap();

    unsafe { fun(name.as_ptr()) }
}

fn initialize<'a>() -> &'a UEVR_FRenderTargetPoolHookFunctions {
    unsafe {
        if STATIC_RENDER_HOOK.is_null() {
            STATIC_RENDER_HOOK = super::API::get().sdk().render_target_pool_hook;
        }

        &*STATIC_RENDER_HOOK
    }
}
