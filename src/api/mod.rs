pub mod object_hook;
pub mod render_hook;
pub mod stereo_hook;
pub mod vr;

use crate::{
    self as rusty_uevr,
    bindings::{
        wchar_t, UEVR_FFieldHandle, UEVR_FPropertyHandle, UEVR_IConsoleObjectHandle,
        UEVR_PluginInitializeParam, UEVR_Quaternionf, UEVR_SDKData, UEVR_SDKFunctions,
        UEVR_UFieldHandle, UEVR_UObjectHandle, UEVR_UStructHandle, UEVR_Vector3f,
    },
    define_object,
    util::encode_wstr,
};

use std::{
    ffi::{c_void, CString},
    iter,
    mem::ManuallyDrop,
    path::PathBuf,
    ptr::{null, null_mut},
    sync::{Arc, LazyLock, Mutex},
};

// TODO: Does this Arc actually achieve anything? Is it needed in a multithreading context?
static INSTANCE: LazyLock<Arc<Mutex<Option<API>>>> = LazyLock::new(|| Arc::new(Mutex::new(None)));

#[derive(Clone)]
pub struct API {
    param: *const UEVR_PluginInitializeParam,
    sdk: *const UEVR_SDKData,
}

unsafe impl Send for API {}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::api::API::get().log_error(format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::api::API::get().log_warn(format!($($arg)*));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::api::API::get().log_info(format!($($arg)*));
    };
}

impl API {
    pub fn initialize(param: *const UEVR_PluginInitializeParam) {
        let mut instance = INSTANCE.lock().unwrap();

        if instance.is_none() {
            *instance = Some(API {
                param: unsafe { &*param },
                sdk: unsafe { &*(&*param).sdk },
            })
        }
    }

    pub fn get() -> Self {
        INSTANCE
            .lock()
            .unwrap()
            .as_ref()
            .expect("tried to access the API before it was initialized")
            .clone()
    }

    pub const fn param(&self) -> &UEVR_PluginInitializeParam {
        unsafe { &*self.param }
    }

    pub const fn sdk(&self) -> &UEVR_SDKData {
        unsafe { &*self.sdk }
    }

    pub const fn functions(&self) -> &UEVR_SDKFunctions {
        unsafe { &*self.sdk().functions }
    }

    pub fn get_persistent_dir(&self) -> PathBuf {
        unsafe {
            let fun = (&*self.param().functions).get_persistent_dir.unwrap();
            let size = fun(null_mut(), 0);
            if size == 0 {
                return PathBuf::new();
            }

            let mut result = vec![0u16; size as usize];
            fun(result.as_mut_ptr(), size + 1);

            PathBuf::from(String::from_utf16_lossy(&result))
        }
    }

    pub fn dispatch_lua_event(&self, event_name: impl AsRef<str>, event_data: impl AsRef<str>) {
        let event_name = CString::new(event_name.as_ref()).unwrap();
        let event_data = CString::new(event_data.as_ref()).unwrap();

        unsafe {
            let fun = (&*self.param().functions).dispatch_lua_event.unwrap();
            fun(event_name.as_ptr(), event_data.as_ptr())
        }
    }

    pub fn log_error(&self, text: String) {
        unsafe {
            println!("[ERROR] {text}");

            let cstr = CString::new(text).unwrap();
            let log_fn = (&*self.param().functions).log_error.unwrap();

            log_fn(cstr.as_ptr());
        }
    }

    pub fn log_warn(&self, text: String) {
        unsafe {
            println!("[WARN] {text}");

            let cstr = CString::new(text).unwrap();
            let log_fn = (&*self.param().functions).log_warn.unwrap();

            log_fn(cstr.as_ptr());
        }
    }

    pub fn log_info(&self, text: String) {
        unsafe {
            println!("[INFO] {text}");

            let cstr = CString::new(text).unwrap();
            let log_fn = (&*self.param().functions).log_info.unwrap();

            log_fn(cstr.as_ptr());
        }
    }

    pub fn find_uobject<T: RUObject>(&self, name: impl AsRef<str>) -> Option<T> {
        unsafe {
            let fun = (&*self.sdk().uobject_array).find_uobject.unwrap();
            let name = encode_wstr(name);
            let ptr = fun(name.as_ptr());

            if ptr.is_null() {
                return None;
            }

            Some(T::from_ptr(ptr as _))
        }
    }

    pub fn get_engine(&self) -> UEngine {
        let fun = self.functions().get_uengine.unwrap();

        unsafe { UEngine::from_ptr(fun() as _) }
    }

    pub fn get_player_controller(&self, index: i32) -> UObject {
        let fun = self.functions().get_player_controller.unwrap();

        unsafe { UObject::from_handle(fun(index)) }
    }

    pub fn get_local_pawn(&self, index: i32) -> UObject {
        let fun = self.functions().get_local_pawn.unwrap();

        unsafe { UObject::from_handle(fun(index)) }
    }

    pub fn spawn_object(&self, class: UClass, outer: UObject) -> UObject {
        let fun = self.functions().spawn_object.unwrap();

        unsafe { UObject::from_handle(fun(class.to_handle(), outer.to_handle())) }
    }

    pub fn execute_command(&self, command: impl AsRef<str>) {
        let fun = self.functions().execute_command.unwrap();
        let command = encode_wstr(command);

        unsafe { fun(command.as_ptr()) }
    }

    pub fn execute_command_ex(
        &self,
        world: UWorld,
        command: impl AsRef<str>,
        output_device: *mut c_void,
    ) {
        let fun = self.functions().execute_command_ex.unwrap();
        let command = encode_wstr(command);

        unsafe { fun(world.to_object_handle(), command.as_ptr(), output_device) }
    }

    pub fn get_uobject_array(&self) -> FUObjectArray {
        let fun = self.functions().get_uobject_array.unwrap();

        unsafe { FUObjectArray::from_handle(fun()) }
    }

    pub fn get_console_manager(&self) -> FConsoleManager {
        let fun = self.functions().get_console_manager.unwrap();

        unsafe { FConsoleManager::from_handle(fun()) }
    }
}

pub trait Ptr {
    fn from_ptr(ptr: *mut c_void) -> Self;
    fn to_ptr(&self) -> *mut c_void;

    fn is_invalid(&self) -> bool {
        self.to_ptr().is_null()
    }

    fn from_ptr_safe(ptr: *mut c_void) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            None
        } else {
            Some(Self::from_ptr(ptr))
        }
    }

    fn cast<T: StaticClass>(&self) -> Option<T>
    where
        Self: StaticClass,
    {
        if self.is_a(T::static_class()) {
            Some(T::from_ptr(self.to_ptr()))
        } else {
            None
        }
    }

    unsafe fn unsafe_cast<T: Ptr>(&self) -> T {
        T::from_ptr(self.to_ptr())
    }
}

pub trait StaticClass: Ptr {
    fn static_class_safe() -> Option<UClass>;

    fn static_class() -> UClass {
        Self::static_class_safe().unwrap()
    }

    fn is_a(&self, cmp: UClass) -> bool {
        unsafe {
            let fun = UObject::initialize().is_a.unwrap();

            fun(self.to_ptr() as *mut _, cmp.to_handle())
        }
    }
}

define_object!(
    FMalloc,
    @functions(UEVR_FMallocHandle, UEVR_FMallocFunctions, malloc)
);

define_object!(
    FName,
    @functions(UEVR_FNameHandle, UEVR_FNameFunctions, fname)
);

define_object!(
    UObject,
    "Object",
    @functions(UEVR_UObjectHandle, UEVR_UObjectFunctions, uobject),
    @class("Class /Script/CoreUObject.Object"),
    @impls(RUObject)
);

define_object!(
    UField,
    "Field",
    @functions(UEVR_UFieldHandle, UEVR_UFieldFunctions, ufield),
    @class("Class /Script/CoreUObject.Field"),
    @impls(RUObject, RUField)
);

define_object!(
    UStruct,
    "Struct",
    @functions(UEVR_UStructHandle, UEVR_UStructFunctions, ustruct),
    @class("Class /Script/CoreUObject.Struct"),
    @impls(RUObject, RUField, RUStruct)
);

define_object!(
    UClass,
    "Class",
    @functions(UEVR_UClassHandle, UEVR_UClassFunctions, uclass),
    @class("Class /Script/CoreUObject.Class"),
    @impls(RUObject, RUField, RUStruct)
);

define_object!(
    UFunction,
    "Function",
    @functions(UEVR_UFunctionHandle, UEVR_UFunctionFunctions, ufunction),
    @class("Class /Script/CoreUObject.Function"),
    @impls(RUObject, RUField, RUStruct)
);

define_object!(
    UScriptStruct,
    "ScriptStruct",
    @functions(UEVR_UScriptStructHandle, UEVR_UScriptStructFunctions, uscriptstruct),
    @class("Class /Script/CoreUObject.ScriptStruct"),
    @impls(RUObject, RUField, RUStruct)
);

define_object!(
    FUObjectArray,
    @functions(UEVR_UObjectArrayHandle, UEVR_UObjectArrayFunctions, uobject_array)
);

define_object!(
    UEnum,
    "Enum",
    @impls(RUObject)
);

define_object!(
    FField,
    @functions(UEVR_FFieldHandle, UEVR_FFieldFunctions, ffield)
);

define_object!(
    FProperty,
    @functions(UEVR_FPropertyHandle, UEVR_FPropertyFunctions, fproperty),
    @impls(RFField)
);

define_object!(
    FArrayProperty,
    @functions(UEVR_FArrayPropertyHandle, UEVR_FArrayPropertyFunctions, farrayproperty),
    @impls(RFField, RFProperty)
);

define_object!(
    FBoolProperty,
    @functions(UEVR_FBoolPropertyHandle, UEVR_FBoolPropertyFunctions, fboolproperty),
    @impls(RFField, RFProperty)
);

define_object!(
    FStructProperty,
    @functions(UEVR_FStructPropertyHandle, UEVR_FStructPropertyFunctions, fstructproperty),
    @impls(RFField, RFProperty)
);

define_object!(
    FNumericProperty,
    @impls(RFField, RFProperty)
);

define_object!(
    FEnumProperty,
    @functions(UEVR_FEnumPropertyHandle, UEVR_FEnumPropertyFunctions, fenumproperty),
    @impls(RFField, RFProperty)
);

define_object!(
    FFieldClass,
    @functions(UEVR_FFieldClassHandle, UEVR_FFieldClassFunctions, ffield_class)
);

define_object!(
    FConsoleManager,
    @functions(UEVR_FConsoleManagerHandle, UEVR_ConsoleFunctions, console)
);

define_object!(
    FRHITexture2D,
    @functions(UEVR_FRHITexture2DHandle, UEVR_FRHITexture2DFunctions, frhitexture2d)
);

define_object!(
    IConsoleObject,
    @functions(UEVR_IConsoleObjectHandle, UEVR_ConsoleFunctions, console)
);

define_object!(
    IConsoleVariable,
    @functions(UEVR_IConsoleVariableHandle, UEVR_ConsoleFunctions, console),
    @impls(RIConsoleObject)
);

define_object!(
    IConsoleCommand,
    @functions(UEVR_IConsoleCommandHandle, UEVR_ConsoleFunctions, console),
    @impls(RIConsoleObject)
);

// TODO
define_object!(
    UEngine,
    "Engine",
    @impls(RUObject)
);

// TODO
define_object!(
    UGameEngine,
    "GameEngine",
    @impls(RUObject) // This should actually implement RUEngine, but since the implementation for that is missing, we implement object instead
);

// TODO
define_object!(
    UWorld,
    "World",
    @class("Class /Script/Engine.World"),
    @impls(RUObject)
);

define_object!(
    MotionControllerState,
    "MotionControllerState",
    @functions(UEVR_UObjectHookMotionControllerStateHandle, UEVR_UObjectHookMotionControllerStateFunctions, [(*(*API::get().sdk()).uobject_hook).mc_state])
);

#[repr(u32)]
pub enum EFindName {
    Find = 0,
    Add = 1,
}

impl FName {
    // Looks like there is no destructor, will this cause memory leaks? Or is this GC'd?
    pub fn new(name: &str, find_type: Option<EFindName>) -> Self {
        let instance = Self(ManuallyDrop::new([0; 8]).as_mut_ptr() as *mut c_void);
        let fun = Self::initialize().constructor.unwrap();

        let name = name.encode_utf16().chain(iter::once(0)).collect::<Vec<_>>();

        unsafe {
            fun(
                instance.to_handle(),
                name.as_ptr(),
                find_type.unwrap_or(EFindName::Add) as u32,
            );
        }

        instance
    }

    pub fn to_string(&self) -> String {
        let fun = Self::initialize().to_string.unwrap();
        let size = unsafe { fun(self.to_handle(), null_mut(), 0) };

        if size == 0 {
            return "".to_string();
        }

        let mut ptr = vec![0u16; size as usize];
        unsafe {
            fun(self.to_handle(), ptr.as_mut_ptr(), size);
        }

        String::from_utf16(&ptr).unwrap()
    }
}

impl FMalloc {
    pub fn get() -> Self {
        let fun = Self::initialize().get.unwrap();

        unsafe { Self::from_handle(fun()) }
    }

    pub unsafe fn malloc(&self, size: u32, alignment: u32) -> *mut c_void {
        let fun = Self::initialize().malloc.unwrap();
        fun(self.to_handle(), size, alignment)
    }

    pub unsafe fn realloc(&self, original: *mut c_void, size: u32, alignment: u32) -> *mut c_void {
        let fun = Self::initialize().realloc.unwrap();
        fun(self.to_handle(), original, size, alignment)
    }

    pub unsafe fn free(&self, original: *mut c_void) {
        let fun = Self::initialize().free.unwrap();
        fun(self.to_handle(), original)
    }
}

pub trait RUObject: Ptr {
    fn to_object_handle(&self) -> UEVR_UObjectHandle {
        self.to_ptr() as _
    }

    fn get_class(&self) -> Option<UClass> {
        let fun = UObject::initialize().get_class.unwrap();

        unsafe { UClass::from_handle_safe(fun(self.to_object_handle())) }
    }

    fn get_outer(&self) -> Option<UObject> {
        let fun = UObject::initialize().get_outer.unwrap();

        unsafe { UObject::from_handle_safe(fun(self.to_object_handle())) }
    }

    fn process_event(&self, function: UFunction, params: *mut c_void) {
        let fun = UObject::initialize().process_event.unwrap();

        unsafe { fun(self.to_object_handle(), function.to_handle(), params) }
    }

    fn call_function(&self, name: &str, params: *mut c_void) {
        let name = name.encode_utf16().chain(iter::once(0)).collect::<Vec<_>>();
        let fun = UObject::initialize().call_function.unwrap();

        unsafe {
            fun(self.to_object_handle(), name.as_ptr(), params);
        }
    }

    fn get_property_data<T>(&self, name: &str) -> *mut T {
        let name = name.encode_utf16().chain(iter::once(0)).collect::<Vec<_>>();
        let fun = UObject::initialize().get_property_data.unwrap();

        unsafe { fun(self.to_object_handle(), name.as_ptr()) as *mut T }
    }

    fn get_property<T>(&self, name: &str) -> &mut T {
        unsafe { &mut *self.get_property_data(name) }
    }

    fn get_bool_property(&self, name: &str) -> bool {
        let name = name.encode_utf16().chain(iter::once(0)).collect::<Vec<_>>();
        let fun = UObject::initialize().get_bool_property.unwrap();

        unsafe { fun(self.to_object_handle(), name.as_ptr()) }
    }

    fn set_bool_property(&self, name: &str, value: bool) {
        let name = name.encode_utf16().chain(iter::once(0)).collect::<Vec<_>>();
        let fun = UObject::initialize().set_bool_property.unwrap();

        unsafe { fun(self.to_object_handle(), name.as_ptr(), value) }
    }

    fn get_fname(&self) -> FName {
        let fun = UObject::initialize().get_fname.unwrap();

        unsafe { FName::from_handle(fun(self.to_object_handle())) }
    }

    fn get_full_name(&self) -> String {
        let Some(class) = self.get_class().and_then(|class| class.cast::<UObject>()) else {
            return "".to_string();
        };

        let mut name = self.get_fname().to_string();
        let mut current = self.get_outer();

        while let Some(outer) = current {
            if std::ptr::addr_eq(outer.to_ptr(), self.to_ptr()) {
                break;
            }

            name = format!("{}.{name}", outer.get_fname().to_string());
            current = outer.get_outer();
        }

        format!("{} {name}", class.get_fname().to_string())
    }
}

pub trait RUField: RUObject {
    fn to_field_handle(&self) -> UEVR_UFieldHandle {
        self.to_ptr() as _
    }

    fn get_next(&self) -> UField {
        let fun = UField::initialize().get_next.unwrap();

        unsafe { UField::from_handle(fun(self.to_field_handle())) }
    }
}

pub trait RUStruct: RUField {
    fn to_struct_handle(&self) -> UEVR_UStructHandle {
        self.to_ptr() as _
    }

    fn get_super_struct(&self) -> UStruct {
        let fun = UStruct::initialize().get_super_struct.unwrap();

        unsafe { UStruct::from_handle(fun(self.to_struct_handle())) }
    }

    fn get_super(&self) -> UStruct {
        self.get_super_struct()
    }

    fn find_function(&self, name: impl AsRef<str>) -> UFunction {
        let name = encode_wstr(name);
        let fun = UStruct::initialize().find_function.unwrap();

        unsafe { UFunction::from_handle(fun(self.to_struct_handle(), name.as_ptr())) }
    }

    fn find_property(&self, name: impl AsRef<str>) -> FProperty {
        let name = encode_wstr(name);
        let fun = UStruct::initialize().find_property.unwrap();

        unsafe { FProperty::from_handle(fun(self.to_struct_handle(), name.as_ptr())) }
    }

    fn get_child_properties(&self) -> FField {
        let fun = UStruct::initialize().get_child_properties.unwrap();

        unsafe { FField::from_handle(fun(self.to_struct_handle())) }
    }

    fn get_children(&self) -> UField {
        let fun = UStruct::initialize().get_children.unwrap();

        unsafe { UField::from_handle(fun(self.to_struct_handle())) }
    }

    fn get_properties_size(&self) -> i32 {
        let fun = UStruct::initialize().get_properties_size.unwrap();

        unsafe { fun(self.to_struct_handle()) }
    }

    fn get_min_alignment(&self) -> i32 {
        let fun = UStruct::initialize().get_min_alignment.unwrap();

        unsafe { fun(self.to_struct_handle()) }
    }
}

impl UClass {
    pub fn get_class_default_object(&self) -> UObject {
        let fun = Self::initialize().get_class_default_object.unwrap();

        unsafe { UObject::from_handle(fun(self.to_handle())) }
    }

    pub fn get_objects_matching<T: StaticClass>(&self, allow_default: bool) -> Vec<T> {
        let objects = self.get_objects_matching_raw(allow_default);

        objects.into_iter().flat_map(|obj| obj.cast()).collect()
    }

    pub unsafe fn get_objects_matching_unsafe<T: Ptr>(&self, allow_default: bool) -> Vec<T> {
        let objects = self.get_objects_matching_raw(allow_default);

        objects.into_iter().map(|obj| obj.unsafe_cast()).collect()
    }

    pub fn get_first_object_matching<T: StaticClass>(&self, allow_default: bool) -> Option<T> {
        let object = self.get_first_object_matching_raw(allow_default);

        object.and_then(|object| object.cast())
    }

    pub unsafe fn get_first_object_matching_unsafe<T: Ptr>(
        &self,
        allow_default: bool,
    ) -> Option<T> {
        let object = self.get_first_object_matching_raw(allow_default);

        object.map(|object| object.unsafe_cast())
    }

    fn get_objects_matching_raw(&self, allow_default: bool) -> Vec<UObject> {
        Self::activate();

        let fun = unsafe {
            (&*API::get().sdk().uobject_hook)
                .get_objects_by_class
                .unwrap()
        };

        let size = unsafe { fun(self.to_handle(), null_mut(), 0, allow_default) };
        if size == 0 {
            return vec![];
        }

        let mut result = Vec::with_capacity(size as _);

        unsafe {
            result.set_len(size as _);

            fun(
                self.to_handle(),
                result.as_mut_ptr(),
                size as u32,
                allow_default,
            );
        }

        result.into_iter().map(UObject::from_handle).collect()
    }

    fn get_first_object_matching_raw(&self, allow_default: bool) -> Option<UObject> {
        Self::activate();

        let fun = unsafe {
            (&*API::get().sdk().uobject_hook)
                .get_first_object_by_class
                .unwrap()
        };

        unsafe { UObject::from_handle_safe(fun(self.to_handle(), allow_default)) }
    }

    fn activate() {
        unsafe {
            let fun = (&*API::get().sdk().uobject_hook).activate.unwrap();

            fun()
        }
    }
}

impl UFunction {
    pub fn call(&self, obj: UObject, params: *mut c_void) {
        if obj.is_invalid() {
            return;
        }

        obj.process_event(*self, params);
    }

    pub fn get_native_function(&self) -> *mut c_void {
        let fun = Self::initialize().get_native_function.unwrap();

        unsafe { fun(self.to_handle()) }
    }

    pub fn get_function_flags(&self) -> u32 {
        let fun = Self::initialize().get_function_flags.unwrap();

        unsafe { fun(self.to_handle()) }
    }

    pub fn set_function_flags(&self, flags: u32) {
        let fun = Self::initialize().set_function_flags.unwrap();

        unsafe { fun(self.to_handle(), flags) }
    }
}

pub struct StructOpts {
    pub size: i32,
    pub alignment: i32,
}

impl UScriptStruct {
    pub fn get_struct_opts(&self) -> &mut StructOpts {
        let fun = Self::initialize().get_struct_ops.unwrap();

        unsafe { (fun(self.to_handle()) as *mut StructOpts).as_mut().unwrap() }
    }

    pub fn get_struct_size(&self) -> i32 {
        let fun = Self::initialize().get_struct_size.unwrap();

        unsafe { fun(self.to_handle()) }
    }
}

pub trait RFField: Ptr {
    fn to_ffield_handle(&self) -> UEVR_FFieldHandle {
        self.to_ptr() as _
    }

    fn get_next(&self) -> Option<FField> {
        let fun = FField::initialize().get_next.unwrap();

        unsafe { FField::from_handle_safe(fun(self.to_ffield_handle())) }
    }

    fn get_fname(&self) -> FName {
        let fun = FField::initialize().get_fname.unwrap();

        unsafe { FName::from_handle(fun(self.to_ffield_handle())) }
    }

    fn get_class(&self) -> FFieldClass {
        let fun = FField::initialize().get_class.unwrap();

        unsafe { FFieldClass::from_handle(fun(self.to_ffield_handle())) }
    }
}

pub trait RFProperty: RFField {
    fn to_fproperty_handle(&self) -> UEVR_FPropertyHandle {
        self.to_ptr() as _
    }

    fn get_offset(&self) -> i32 {
        let fun = FProperty::initialize().get_offset.unwrap();

        unsafe { fun(self.to_fproperty_handle()) }
    }

    fn get_property_flags(&self) -> u64 {
        let fun = FProperty::initialize().get_property_flags.unwrap();

        unsafe { fun(self.to_fproperty_handle()) }
    }

    fn is_param(&self) -> bool {
        let fun = FProperty::initialize().is_param.unwrap();

        unsafe { fun(self.to_fproperty_handle()) }
    }

    fn is_out_param(&self) -> bool {
        let fun = FProperty::initialize().is_out_param.unwrap();

        unsafe { fun(self.to_fproperty_handle()) }
    }

    fn is_return_param(&self) -> bool {
        let fun = FProperty::initialize().is_return_param.unwrap();

        unsafe { fun(self.to_fproperty_handle()) }
    }

    fn is_reference_param(&self) -> bool {
        let fun = FProperty::initialize().is_reference_param.unwrap();

        unsafe { fun(self.to_fproperty_handle()) }
    }

    fn is_pod(&self) -> bool {
        let fun = FProperty::initialize().is_pod.unwrap();

        unsafe { fun(self.to_fproperty_handle()) }
    }
}

impl FArrayProperty {
    pub fn get_inner(&self) -> FProperty {
        let fun = Self::initialize().get_inner.unwrap();

        unsafe { FProperty::from_handle(fun(self.to_handle())) }
    }
}

impl FBoolProperty {
    pub fn get_field_size(&self) -> u32 {
        let fun = Self::initialize().get_field_size.unwrap();

        unsafe { fun(self.to_handle()) }
    }

    pub fn get_byte_offset(&self) -> u32 {
        let fun = Self::initialize().get_byte_offset.unwrap();

        unsafe { fun(self.to_handle()) }
    }

    pub fn get_byte_mask(&self) -> u32 {
        let fun = Self::initialize().get_byte_mask.unwrap();

        unsafe { fun(self.to_handle()) }
    }

    pub fn get_field_mask(&self) -> u32 {
        let fun = Self::initialize().get_field_mask.unwrap();

        unsafe { fun(self.to_handle()) }
    }

    pub fn get_value_from_object(&self, object: *mut c_void) -> bool {
        let fun = Self::initialize().get_value_from_object.unwrap();

        unsafe { fun(self.to_handle(), object) }
    }

    pub fn get_value_from_propbase(&self, addr: *mut c_void) -> bool {
        let fun = Self::initialize().get_value_from_propbase.unwrap();

        unsafe { fun(self.to_handle(), addr) }
    }

    pub fn set_value_in_object(&self, object: *mut c_void, value: bool) {
        let fun = Self::initialize().set_value_in_object.unwrap();

        unsafe { fun(self.to_handle(), object, value) }
    }

    pub fn set_value_in_propbase(&self, addr: *mut c_void, value: bool) {
        let fun = Self::initialize().set_value_in_propbase.unwrap();

        unsafe { fun(self.to_handle(), addr, value) }
    }
}

impl FStructProperty {
    pub fn get_struct(&self) -> UScriptStruct {
        let fun = Self::initialize().get_struct.unwrap();

        unsafe { UScriptStruct::from_handle(fun(self.to_handle())) }
    }
}

impl FEnumProperty {
    pub fn get_underlying_prop(&self) -> FNumericProperty {
        let fun = Self::initialize().get_underlying_prop.unwrap();

        unsafe { FNumericProperty::from_ptr(fun(self.to_handle()) as _) }
    }

    pub fn get_enum(&self) -> UEnum {
        let fun = Self::initialize().get_enum.unwrap();

        unsafe { UEnum::from_ptr(fun(self.to_handle()) as _) }
    }
}

impl FFieldClass {
    pub fn get_fname(&self) -> FName {
        let fun = Self::initialize().get_fname.unwrap();

        unsafe { FName::from_handle(fun(self.to_handle())) }
    }

    pub fn get_name(&self) -> String {
        self.get_fname().to_string()
    }
}

#[repr(C)]
pub struct ConsoleObjectElement {
    key: *mut wchar_t,
    unk: [i32; 2],
    value: *mut IConsoleObject,
    unk2: [i32; 2],
}

// TODO: If there's no need to use TArray anywhere else, we can just instantly convert to a Vec and free the original memory
impl FConsoleManager {
    pub fn get_console_objects(&self) -> TArray<ConsoleObjectElement> {
        let fun = Self::initialize().get_console_objects.unwrap();

        unsafe { (&*(fun(self.to_handle()) as *const TArray<ConsoleObjectElement>)).clone() }
    }

    pub fn find_object(&self, name: impl AsRef<str>) -> IConsoleObject {
        let name = encode_wstr(name);
        let fun = Self::initialize().find_object.unwrap();

        unsafe { IConsoleObject::from_handle(fun(self.to_handle(), name.as_ptr())) }
    }

    pub fn find_variable(&self, name: impl AsRef<str>) -> IConsoleVariable {
        let name = encode_wstr(name);
        let fun = Self::initialize().find_variable.unwrap();

        unsafe { IConsoleVariable::from_handle(fun(self.to_handle(), name.as_ptr())) }
    }

    pub fn find_command(&self, name: impl AsRef<str>) -> IConsoleCommand {
        let name = encode_wstr(name);
        let fun = Self::initialize().find_command.unwrap();

        unsafe { IConsoleCommand::from_handle(fun(self.to_handle(), name.as_ptr())) }
    }
}

pub trait RIConsoleObject: Ptr {
    fn to_iconsole_handle(&self) -> UEVR_IConsoleObjectHandle {
        self.to_ptr() as _
    }

    fn as_command(&self) -> IConsoleCommand {
        let fun = IConsoleObject::initialize().as_command.unwrap();

        unsafe { IConsoleCommand::from_handle(fun(self.to_iconsole_handle())) }
    }
}

impl IConsoleVariable {
    pub fn set(&self, value: impl AsRef<str>) {
        let value = encode_wstr(value);
        let fun = Self::initialize().variable_set.unwrap();

        unsafe { fun(self.to_handle(), value.as_ptr()) }
    }

    pub fn set_ex(&self, value: impl AsRef<str>, flags: Option<u32>) {
        let value = encode_wstr(value);
        let flags = flags.unwrap_or(0x80000000);
        let fun = Self::initialize().variable_set_ex.unwrap();

        unsafe { fun(self.to_handle(), value.as_ptr(), flags) }
    }

    pub fn get_int(&self) -> i32 {
        let fun = Self::initialize().variable_get_int.unwrap();

        unsafe { fun(self.to_handle()) }
    }

    pub fn get_float(&self) -> f32 {
        let fun = Self::initialize().variable_get_float.unwrap();

        unsafe { fun(self.to_handle()) }
    }
}

impl IConsoleCommand {
    pub fn execute(&self, args: impl AsRef<str>) {
        let args = encode_wstr(args);
        let fun = Self::initialize().command_execute.unwrap();

        unsafe { fun(self.to_handle(), args.as_ptr()) }
    }
}

impl UEngine {
    pub fn get() -> UEngine {
        API::get().get_engine()
    }
}

#[derive(Clone, Copy)]
pub struct FUObjectItem {
    pub object: UEVR_UObjectHandle,
    pub flags: i32,
    pub cluster_index: i32,
    pub serial_number: i32,
}

impl FUObjectArray {
    pub fn get() -> FUObjectArray {
        API::get().get_uobject_array()
    }

    pub fn is_chunked() -> bool {
        let fun = Self::initialize().is_chunked.unwrap();

        unsafe { fun() }
    }

    pub fn is_inlined() -> bool {
        let fun = Self::initialize().is_inlined.unwrap();

        unsafe { fun() }
    }

    pub fn get_objects_offset() -> u64 {
        let fun = Self::initialize().get_objects_offset.unwrap();

        unsafe { fun() as _ }
    }

    pub fn get_item_distance() -> u64 {
        let fun = Self::initialize().get_item_distance.unwrap();

        unsafe { fun() as _ }
    }

    pub fn get_object_count(&self) -> i32 {
        let fun = Self::initialize().get_object_count.unwrap();

        unsafe { fun(self.to_handle()) }
    }

    pub fn get_objects_ptr(&self) -> *mut c_void {
        let fun = Self::initialize().get_objects_ptr.unwrap();

        unsafe { fun(self.to_handle()) }
    }

    pub fn get_object(&self, index: i32) -> UObject {
        let fun = Self::initialize().get_object.unwrap();

        unsafe { UObject::from_handle(fun(self.to_handle(), index)) }
    }

    pub fn get_item(&self, index: i32) -> &FUObjectItem {
        let fun = Self::initialize().get_item.unwrap();

        unsafe { &*(fun(self.to_handle(), index) as *const FUObjectItem) }
    }
}

impl FRHITexture2D {
    pub fn get_native_resource(&self) -> *mut c_void {
        let fun = Self::initialize().get_native_resource.unwrap();

        unsafe { fun(self.to_handle()) }
    }
}

impl MotionControllerState {
    pub fn set_rotation_offset(&self, offset: *const UEVR_Quaternionf) {
        let fun = Self::initialize().set_rotation_offset.unwrap();

        unsafe { fun(self.to_handle(), offset) }
    }

    pub fn set_location_offset(&self, offset: *const UEVR_Vector3f) {
        let fun = Self::initialize().set_location_offset.unwrap();

        unsafe { fun(self.to_handle(), offset) }
    }

    pub fn set_hand(&self, hand: u32) {
        let fun = Self::initialize().set_hand.unwrap();

        unsafe { fun(self.to_handle(), hand) }
    }

    pub fn set_permanent(&self, permanent: bool) {
        let fun = Self::initialize().set_permanent.unwrap();

        unsafe { fun(self.to_handle(), permanent) }
    }
}

pub struct TArray<T> {
    data: *mut T,
    count: i32,
    capacity: i32,
}

impl<T> TArray<T> {
    pub fn begin(&self) -> *const T {
        self.data
    }

    pub fn begin_mut(&mut self) -> *mut T {
        self.data
    }

    pub fn end(&self) -> *const T {
        if self.data.is_null() {
            null()
        } else {
            unsafe { self.data.byte_add(self.count as _) }
        }
    }

    pub fn end_mut(&mut self) -> *mut T {
        self.end() as _
    }

    pub fn empty(&self) -> bool {
        self.count == 0 || self.data.is_null()
    }

    pub unsafe fn to_vec(self) -> Vec<T> {
        Vec::from_raw_parts(self.data, self.count as _, self.capacity as _)
    }
}

impl<T> Clone for TArray<T> {
    fn clone(&self) -> Self {
        TArray {
            data: self.data,
            capacity: self.capacity,
            count: self.count,
        }
    }
}

impl<T> Drop for TArray<T> {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                FMalloc::get().free(self.data as _);
            }
        }
    }
}
