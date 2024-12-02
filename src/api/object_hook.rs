use crate::{
    api::{MotionControllerState, UClass, UObject},
    bindings::UEVR_UObjectHookFunctions,
};

use std::ptr::null;

static mut STATIC_OBJECT_HOOK: *const UEVR_UObjectHookFunctions = null();

pub fn activate() {
    let fun = initialize().activate.unwrap();

    unsafe { fun() }
}

pub fn exists(obj: UObject) -> bool {
    let fun = initialize().exists.unwrap();

    unsafe { fun(obj.to_handle()) }
}

pub fn is_disabled() -> bool {
    let fun = initialize().is_disabled.unwrap();

    unsafe { fun() }
}

pub fn set_disabled(disabled: bool) {
    let fun = initialize().set_disabled.unwrap();

    unsafe { fun(disabled) }
}

pub fn get_objects_by_class(c: UClass, allow_default: bool) -> Vec<UObject> {
    c.get_objects_matching_raw(allow_default)
}

pub fn get_first_object_by_class(c: UClass, allow_default: bool) -> Option<UObject> {
    c.get_first_object_matching_raw(allow_default)
}

pub fn get_or_add_motion_controller_state(obj: UObject) -> MotionControllerState {
    let fun = initialize().get_or_add_motion_controller_state.unwrap();

    unsafe { MotionControllerState::from_handle(fun(obj.to_handle())) }
}

pub fn get_motion_controller_state(obj: UObject) -> MotionControllerState {
    let fun = initialize().get_motion_controller_state.unwrap();

    unsafe { MotionControllerState::from_handle(fun(obj.to_handle())) }
}

pub fn remove_motion_controller_state(obj: UObject) {
    let fun = initialize().remove_motion_controller_state.unwrap();

    unsafe { fun(obj.to_handle()) }
}

pub fn remove_all_motion_controller_states() {
    let fun = initialize().remove_all_motion_controller_states.unwrap();

    unsafe { fun() }
}

fn initialize<'a>() -> &'a UEVR_UObjectHookFunctions {
    unsafe {
        if STATIC_OBJECT_HOOK.is_null() {
            STATIC_OBJECT_HOOK = super::API::get().sdk().uobject_hook;
        }

        &*STATIC_OBJECT_HOOK
    }
}
