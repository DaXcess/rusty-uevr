use crate::bindings::{
    UEVR_ActionHandle, UEVR_InputSourceHandle, UEVR_Matrix4x4f, UEVR_Quaternionf,
    UEVR_TrackedDeviceIndex, UEVR_VRData, UEVR_Vector2f, UEVR_Vector3f,
};

use std::{
    ffi::{CStr, CString},
    mem::{transmute, zeroed},
    ptr::null,
};

static mut STATIC_UEVR_VRDATA: *const UEVR_VRData = null();

pub trait ModValue {
    fn serialize(self) -> CString;
    fn deserialize(value: &CStr) -> Self;
}

impl ModValue for String {
    fn deserialize(value: &CStr) -> Self {
        value.to_string_lossy().to_string()
    }

    fn serialize(self) -> CString {
        CString::new(self).unwrap()
    }
}

impl ModValue for bool {
    fn deserialize(value: &CStr) -> Self {
        value.to_string_lossy().to_string() == "true"
    }

    fn serialize(self) -> CString {
        if self {
            CString::new("true").unwrap()
        } else {
            CString::new("false").unwrap()
        }
    }
}

pub struct Pose {
    position: UEVR_Vector3f,
    rotation: UEVR_Quaternionf,
}

#[repr(i32)]
pub enum Eye {
    Left,
    Right,
}

#[repr(i32)]
pub enum AimMethod {
    Game,
    Head,
    RightController,
    LeftController,
    TwoHandedRight,
    TwoHandedLeft,
}

pub fn is_runtime_ready() -> bool {
    let fun = initialize().is_runtime_ready.unwrap();

    unsafe { fun() }
}

pub fn is_openvr() -> bool {
    let fun = initialize().is_openvr.unwrap();

    unsafe { fun() }
}

pub fn is_openxr() -> bool {
    let fun = initialize().is_openxr.unwrap();

    unsafe { fun() }
}

pub fn is_hmd_active() -> bool {
    let fun = initialize().is_hmd_active.unwrap();

    unsafe { fun() }
}

pub fn get_standing_origin() -> UEVR_Vector3f {
    let fun = initialize().get_standing_origin.unwrap();
    let mut result = unsafe { zeroed() };

    unsafe { fun(&mut result) }
    result
}

pub fn get_rotation_offset() -> UEVR_Quaternionf {
    let fun = initialize().get_rotation_offset.unwrap();
    let mut result = unsafe { zeroed() };

    unsafe { fun(&mut result) }
    result
}

pub fn set_standing_origin(origin: &UEVR_Vector3f) {
    let fun = initialize().set_standing_origin.unwrap();

    unsafe { fun(origin) }
}

pub fn set_rotation_offset(offset: &UEVR_Quaternionf) {
    let fun = initialize().set_rotation_offset.unwrap();

    unsafe { fun(offset) }
}

pub fn get_hmd_index() -> UEVR_TrackedDeviceIndex {
    let fun = initialize().get_hmd_index.unwrap();

    unsafe { fun() }
}

pub fn get_left_controller_index() -> UEVR_TrackedDeviceIndex {
    let fun = initialize().get_left_controller_index.unwrap();

    unsafe { fun() }
}

pub fn get_right_controller_index() -> UEVR_TrackedDeviceIndex {
    let fun = initialize().get_right_controller_index.unwrap();

    unsafe { fun() }
}

pub fn get_pose(index: UEVR_TrackedDeviceIndex) -> Pose {
    let fun = initialize().get_pose.unwrap();
    let mut result = unsafe { zeroed::<Pose>() };

    unsafe { fun(index, &mut result.position, &mut result.rotation) }
    result
}

pub fn get_transform(index: UEVR_TrackedDeviceIndex) -> UEVR_Matrix4x4f {
    let fun = initialize().get_transform.unwrap();
    let mut result = unsafe { zeroed() };

    unsafe { fun(index, &mut result) }
    result
}

pub fn get_grip_pose(index: UEVR_TrackedDeviceIndex) -> Pose {
    let fun = initialize().get_grip_pose.unwrap();
    let mut result = unsafe { zeroed::<Pose>() };

    unsafe { fun(index, &mut result.position, &mut result.rotation) }
    result
}

pub fn get_aim_pose(index: UEVR_TrackedDeviceIndex) -> Pose {
    let fun = initialize().get_aim_pose.unwrap();
    let mut result = unsafe { zeroed::<Pose>() };

    unsafe { fun(index, &mut result.position, &mut result.rotation) }
    result
}

pub fn get_grip_transform(index: UEVR_TrackedDeviceIndex) -> UEVR_Matrix4x4f {
    let fun = initialize().get_grip_transform.unwrap();
    let mut result = unsafe { zeroed() };

    unsafe { fun(index, &mut result) }
    result
}

pub fn get_aim_transform(index: UEVR_TrackedDeviceIndex) -> UEVR_Matrix4x4f {
    let fun = initialize().get_aim_transform.unwrap();
    let mut result = unsafe { zeroed() };

    unsafe { fun(index, &mut result) }
    result
}

pub fn get_eye_offset(eye: Eye) -> UEVR_Vector3f {
    let fun = initialize().get_eye_offset.unwrap();
    let mut result = unsafe { zeroed() };

    unsafe { fun(eye as i32, &mut result) }
    result
}

pub fn get_ue_projection_matrix(eye: Eye) -> UEVR_Matrix4x4f {
    let fun = initialize().get_ue_projection_matrix.unwrap();
    let mut result = unsafe { zeroed() };

    unsafe { fun(eye as i32, &mut result) }
    result
}

pub fn get_left_joystick_source() -> UEVR_InputSourceHandle {
    let fun = initialize().get_left_joystick_source.unwrap();

    unsafe { fun() }
}

pub fn get_right_joystick_source() -> UEVR_InputSourceHandle {
    let fun = initialize().get_right_joystick_source.unwrap();

    unsafe { fun() }
}

pub fn get_action_handle(name: impl AsRef<str>) -> UEVR_ActionHandle {
    let fun = initialize().get_action_handle.unwrap();
    let name = CString::new(name.as_ref()).unwrap();

    unsafe { fun(name.as_ptr()) }
}

pub fn is_action_active(handle: UEVR_ActionHandle, source: UEVR_InputSourceHandle) -> bool {
    let fun = initialize().is_action_active.unwrap();

    unsafe { fun(handle, source) }
}

pub fn is_action_active_any_joystick(handle: UEVR_ActionHandle) -> bool {
    let fun = initialize().is_action_active_any_joystick.unwrap();

    unsafe { fun(handle) }
}

pub fn get_joystick_axis(source: UEVR_InputSourceHandle) -> UEVR_Vector2f {
    let fun = initialize().get_joystick_axis.unwrap();
    let mut result = unsafe { zeroed() };

    unsafe { fun(source, &mut result) }
    result
}

pub fn trigger_haptic_vibration(
    delay: f32,
    amplitude: f32,
    frequency: f32,
    duration: f32,
    source: UEVR_InputSourceHandle,
) {
    let fun = initialize().trigger_haptic_vibration.unwrap();

    unsafe { fun(delay, amplitude, frequency, duration, source) }
}

pub fn is_using_controllers() -> bool {
    let fun = initialize().is_using_controllers.unwrap();

    unsafe { fun() }
}

pub fn get_movement_orientation() -> AimMethod {
    let fun = initialize().get_movement_orientation.unwrap();

    unsafe { transmute(fun()) }
}

pub fn get_lowest_xinput_index() -> u32 {
    let fun = initialize().get_lowest_xinput_index.unwrap();

    unsafe { fun() }
}

pub fn recenter_view() {
    let fun = initialize().recenter_view.unwrap();

    unsafe { fun() }
}

pub fn recenter_horizon() {
    let fun = initialize().recenter_horizon.unwrap();

    unsafe { fun() }
}

pub fn get_aim_method() -> AimMethod {
    let fun = initialize().get_aim_method.unwrap();

    unsafe { transmute(fun()) }
}

pub fn set_aim_method(method: AimMethod) {
    let fun = initialize().set_aim_method.unwrap();

    unsafe { fun(method as u32) }
}

pub fn is_aim_allowed() -> bool {
    let fun = initialize().is_aim_allowed.unwrap();

    unsafe { fun() }
}

pub fn set_aim_allowed(allowed: bool) {
    let fun = initialize().set_aim_allowed.unwrap();

    unsafe { fun(allowed) }
}

pub fn get_hmd_width() -> u32 {
    let fun = initialize().get_hmd_width.unwrap();

    unsafe { fun() }
}

pub fn get_hmd_height() -> u32 {
    let fun = initialize().get_hmd_height.unwrap();

    unsafe { fun() }
}

pub fn get_ui_width() -> u32 {
    let fun = initialize().get_ui_width.unwrap();

    unsafe { fun() }
}

pub fn get_ui_height() -> u32 {
    let fun = initialize().get_ui_height.unwrap();

    unsafe { fun() }
}

pub fn is_snap_turn_enabled() -> bool {
    let fun = initialize().is_snap_turn_enabled.unwrap();

    unsafe { fun() }
}

pub fn set_snap_turn_enabled(enabled: bool) {
    let fun = initialize().set_snap_turn_enabled.unwrap();

    unsafe { fun(enabled) }
}

pub fn is_decoupled_pitch_enabled() -> bool {
    let fun = initialize().is_decoupled_pitch_enabled.unwrap();

    unsafe { fun() }
}

pub fn set_decoupled_pitch_enabled(enabled: bool) {
    let fun = initialize().set_decoupled_pitch_enabled.unwrap();

    unsafe { fun(enabled) }
}

pub fn set_mod_value<T: ModValue>(key: impl AsRef<str>, value: T) {
    let fun = initialize().set_mod_value.unwrap();
    let key = CString::new(key.as_ref()).unwrap();

    unsafe { fun(key.as_ptr(), value.serialize().as_ptr()) }
}

pub fn get_mod_value<T: ModValue>(key: impl AsRef<str>) -> T {
    let fun = initialize().get_mod_value.unwrap();
    let key = CString::new(key.as_ref()).unwrap();
    let mut result = [0; 256];

    let str = unsafe {
        fun(key.as_ptr(), result.as_mut_ptr(), 256);
        CStr::from_ptr(result.as_ptr())
    };

    T::deserialize(str)
}

pub fn save_config() {
    let fun = initialize().save_config.unwrap();

    unsafe { fun() }
}

pub fn reload_config() {
    let fun = initialize().reload_config.unwrap();

    unsafe { fun() }
}

fn initialize<'a>() -> &'a UEVR_VRData {
    unsafe {
        if STATIC_UEVR_VRDATA.is_null() {
            STATIC_UEVR_VRDATA = super::API::get().param().vr;
        }

        &*STATIC_UEVR_VRDATA
    }
}
