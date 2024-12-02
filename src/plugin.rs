use std::ffi::c_void;

use windows::Win32::{
    Foundation::HWND,
    Graphics::{
        Direct3D11::{ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D},
        Direct3D12::{ID3D12GraphicsCommandList, ID3D12Resource, D3D12_CPU_DESCRIPTOR_HANDLE},
    },
    UI::Input::XboxController::{XINPUT_STATE, XINPUT_VIBRATION},
};

use super::{
    api::{Ptr, UGameEngine},
    bindings::{
        UEVR_FCanvasHandle, UEVR_FSlateRHIRendererHandle, UEVR_FViewportHandle,
        UEVR_FViewportInfoHandle, UEVR_PluginCallbacks, UEVR_Rotatorf, UEVR_SDKCallbacks,
        UEVR_StereoRenderingDeviceHandle, UEVR_UGameEngineHandle, UEVR_UGameViewportClientHandle,
        UEVR_Vector3f,
    },
};

pub static mut _GLOBAL_PLUGIN: Option<Box<dyn Plugin>> = None;

#[allow(unused_variables)]
pub trait Plugin {
    // Main plugin callbacks
    fn on_dllmain(&self) {}
    fn on_initialize(&self) {}
    fn on_present(&self) {}
    fn on_post_render_vr_framework_dx11(
        &self,
        context: *mut ID3D11DeviceContext,
        texture: *mut ID3D11Texture2D,
        rtv: *mut ID3D11RenderTargetView,
    ) {
    }
    fn on_post_render_vr_framework_dx12(
        &self,
        command_list: *mut ID3D12GraphicsCommandList,
        rt: *mut ID3D12Resource,
        rtv: *mut D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
    }
    fn on_device_reset(&self) {}
    fn on_message(&self, hwnd: HWND, msg: u32, wparam: u64, lparam: i64) -> bool {
        true
    }
    fn on_xinput_get_state(&self, retval: &mut u32, user_index: u32, state: *mut XINPUT_STATE) {}
    fn on_xinput_set_state(
        &self,
        retval: &mut u32,
        user_index: u32,
        vibration: *mut XINPUT_VIBRATION,
    ) {
    }

    // Game/Engine callbacks
    fn on_pre_engine_tick(&self, engine: UGameEngine, delta: f32) {}
    fn on_post_engine_tick(&self, engine: UGameEngine, delta: f32) {}
    fn on_pre_slate_draw_window(
        &self,
        renderer: UEVR_FSlateRHIRendererHandle,
        viewport_info: UEVR_FViewportInfoHandle,
    ) {
    }
    fn on_post_slate_draw_window(
        &self,
        renderer: UEVR_FSlateRHIRendererHandle,
        viewport_info: UEVR_FViewportInfoHandle,
    ) {
    }
    fn on_pre_calculate_stereo_view_offset(
        &self,
        device: UEVR_StereoRenderingDeviceHandle,
        view_index: i32,
        world_to_meters: f32,
        position: &mut UEVR_Vector3f,
        rotation: &mut UEVR_Rotatorf,
        is_double: bool,
    ) {
    }
    fn on_post_calculate_stereo_view_offset(
        &self,
        device: UEVR_StereoRenderingDeviceHandle,
        view_index: i32,
        world_to_meters: f32,
        position: &mut UEVR_Vector3f,
        rotation: &mut UEVR_Rotatorf,
        is_double: bool,
    ) {
    }
    fn on_pre_viewport_client_draw(
        &self,
        viewport_client: UEVR_UGameViewportClientHandle,
        viewport: UEVR_FViewportHandle,
        canvas: UEVR_FCanvasHandle,
    ) {
    }
    fn on_post_viewport_client_draw(
        &self,
        viewport_client: UEVR_UGameViewportClientHandle,
        viewport: UEVR_FViewportHandle,
        canvas: UEVR_FCanvasHandle,
    ) {
    }
}

pub unsafe fn setup_callbacks(
    callbacks: *const UEVR_PluginCallbacks,
    sdk_callbacks: *const UEVR_SDKCallbacks,
) {
    let callbacks = &*callbacks;
    let sdk_callbacks = &*sdk_callbacks;

    callbacks.on_device_reset.unwrap_unchecked()(Some(on_device_reset));
    callbacks.on_present.unwrap_unchecked()(Some(on_present));
    callbacks
        .on_post_render_vr_framework_dx11
        .unwrap_unchecked()(Some(on_post_render_vr_framework_dx11));
    callbacks
        .on_post_render_vr_framework_dx12
        .unwrap_unchecked()(Some(on_post_render_vr_framework_dx12));
    callbacks.on_message.unwrap_unchecked()(Some(on_message));
    callbacks.on_xinput_get_state.unwrap()(Some(on_xinput_get_state));
    callbacks.on_xinput_set_state.unwrap()(Some(on_xinput_set_state));

    sdk_callbacks.on_pre_engine_tick.unwrap()(Some(on_pre_engine_tick));
    sdk_callbacks.on_post_engine_tick.unwrap()(Some(on_post_engine_tick));
    sdk_callbacks
        .on_pre_slate_draw_window_render_thread
        .unwrap()(Some(on_pre_slate_draw_window_render_thread));
    sdk_callbacks
        .on_post_slate_draw_window_render_thread
        .unwrap()(Some(on_post_slate_draw_window_render_thread));
    sdk_callbacks.on_pre_calculate_stereo_view_offset.unwrap()(Some(
        on_pre_calculate_stereo_view_offset,
    ));
    sdk_callbacks.on_post_calculate_stereo_view_offset.unwrap()(Some(
        on_post_calculate_stereo_view_offset,
    ));
    sdk_callbacks.on_pre_viewport_client_draw.unwrap()(Some(on_pre_viewport_client_draw));
    sdk_callbacks.on_post_viewport_client_draw.unwrap()(Some(on_post_viewport_client_draw));
}

unsafe extern "C" fn on_device_reset() {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_device_reset();
    }
}

unsafe extern "C" fn on_present() {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_present();
    }
}

unsafe extern "C" fn on_post_render_vr_framework_dx11(
    context: *mut c_void,
    texture: *mut c_void,
    rtv: *mut c_void,
) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_post_render_vr_framework_dx11(
            context as *mut ID3D11DeviceContext,
            texture as *mut ID3D11Texture2D,
            rtv as *mut ID3D11RenderTargetView,
        );
    }
}

unsafe extern "C" fn on_post_render_vr_framework_dx12(
    command_list: *mut c_void,
    rt: *mut c_void,
    rtv: *mut c_void,
) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_post_render_vr_framework_dx12(
            command_list as *mut ID3D12GraphicsCommandList,
            rt as *mut ID3D12Resource,
            rtv as *mut D3D12_CPU_DESCRIPTOR_HANDLE,
        );
    }
}

unsafe extern "C" fn on_message(hwnd: *mut c_void, msg: u32, wparam: u64, lparam: i64) -> bool {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        return plugin.on_message(HWND(hwnd), msg, wparam, lparam);
    }

    true
}

unsafe extern "C" fn on_xinput_get_state(retval: *mut u32, user_index: u32, state: *mut c_void) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_xinput_get_state(
            retval.as_mut().unwrap(),
            user_index,
            state as *mut XINPUT_STATE,
        );
    }
}

unsafe extern "C" fn on_xinput_set_state(
    retval: *mut u32,
    user_index: u32,
    vibration: *mut c_void,
) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_xinput_set_state(
            retval.as_mut().unwrap(),
            user_index,
            vibration as *mut XINPUT_VIBRATION,
        );
    }
}

unsafe extern "C" fn on_pre_engine_tick(engine: UEVR_UGameEngineHandle, delta: f32) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_pre_engine_tick(UGameEngine::from_ptr(engine as *mut c_void), delta);
    }
}

unsafe extern "C" fn on_post_engine_tick(engine: UEVR_UGameEngineHandle, delta: f32) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_post_engine_tick(UGameEngine::from_ptr(engine as *mut c_void), delta);
    }
}

unsafe extern "C" fn on_pre_slate_draw_window_render_thread(
    renderer: UEVR_FSlateRHIRendererHandle,
    viewport_info: UEVR_FViewportInfoHandle,
) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_pre_slate_draw_window(renderer, viewport_info);
    }
}

unsafe extern "C" fn on_post_slate_draw_window_render_thread(
    renderer: UEVR_FSlateRHIRendererHandle,
    viewport_info: UEVR_FViewportInfoHandle,
) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_post_slate_draw_window(renderer, viewport_info);
    }
}

unsafe extern "C" fn on_pre_calculate_stereo_view_offset(
    device: UEVR_StereoRenderingDeviceHandle,
    view_index: i32,
    world_to_meters: f32,
    position: *mut UEVR_Vector3f,
    rotation: *mut UEVR_Rotatorf,
    is_double: bool,
) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_pre_calculate_stereo_view_offset(
            device,
            view_index,
            world_to_meters,
            position.as_mut().unwrap(),
            rotation.as_mut().unwrap(),
            is_double,
        );
    }
}

unsafe extern "C" fn on_post_calculate_stereo_view_offset(
    device: UEVR_StereoRenderingDeviceHandle,
    view_index: i32,
    world_to_meters: f32,
    position: *mut UEVR_Vector3f,
    rotation: *mut UEVR_Rotatorf,
    is_double: bool,
) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_post_calculate_stereo_view_offset(
            device,
            view_index,
            world_to_meters,
            position.as_mut().unwrap(),
            rotation.as_mut().unwrap(),
            is_double,
        );
    }
}

unsafe extern "C" fn on_pre_viewport_client_draw(
    viewport_client: UEVR_UGameViewportClientHandle,
    viewport: UEVR_FViewportHandle,
    canvas: UEVR_FCanvasHandle,
) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_pre_viewport_client_draw(viewport_client, viewport, canvas);
    }
}

unsafe extern "C" fn on_post_viewport_client_draw(
    viewport_client: UEVR_UGameViewportClientHandle,
    viewport: UEVR_FViewportHandle,
    canvas: UEVR_FCanvasHandle,
) {
    if let Some(plugin) = _GLOBAL_PLUGIN.as_ref() {
        plugin.on_post_viewport_client_draw(viewport_client, viewport, canvas);
    }
}
