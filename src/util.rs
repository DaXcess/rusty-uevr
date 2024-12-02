use std::ffi::CString;

/// Simple helper function that converts a string into a CString
///
/// Will panic if the input string contains an internal 0 byte
pub fn encode_cstr(val: impl AsRef<str>) -> CString {
    CString::new(val.as_ref()).unwrap()
}

/// Simple helper function that converts a string into a wide string (Vec<16>)
pub fn encode_wstr(val: impl AsRef<str>) -> Vec<u16> {
    val.as_ref()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}
