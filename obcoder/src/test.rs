use std::ffi::{CStr, CString};
use crate::{avcodec_find_encoder_by_name,get_encoder};

#[test]
fn test() {
    if let Ok(name) = CString::new("h264_nvenc") {
        unsafe {
            let codec = avcodec_find_encoder_by_name(name.as_ptr());
            let codec_name = CStr::from_ptr((*codec).name).to_string_lossy().into_owned();
            println!("{:?}", codec_name);
        }
    };
}

#[test]
fn test_codec(){
    unsafe {
        let codec = get_encoder();
        let codec_name = CStr::from_ptr((*codec).name).to_string_lossy().into_owned();
        println!("{:?}", codec_name);
    }
}