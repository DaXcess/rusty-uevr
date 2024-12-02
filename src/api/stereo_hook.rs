use crate::{api::FRHITexture2D, bindings::UEVR_FFakeStereoRenderingHookFunctions};

use std::ptr::null;

static mut STATIC_STEREO_HOOK: *const UEVR_FFakeStereoRenderingHookFunctions = null();

pub fn get_scene_render_target() -> FRHITexture2D {
    let fun = initialize().get_scene_render_target.unwrap();

    unsafe { FRHITexture2D::from_handle(fun()) }
}

pub fn get_ui_render_target() -> FRHITexture2D {
    let fun = initialize().get_ui_render_target.unwrap();

    unsafe { FRHITexture2D::from_handle(fun()) }
}

fn initialize<'a>() -> &'a UEVR_FFakeStereoRenderingHookFunctions {
    unsafe {
        if STATIC_STEREO_HOOK.is_null() {
            STATIC_STEREO_HOOK = super::API::get().sdk().stereo_hook;
        }

        &*STATIC_STEREO_HOOK
    }
}
