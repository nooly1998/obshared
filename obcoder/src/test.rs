use std::ffi::{CStr, CString};
use crate::{avcodec_find_encoder_by_name, destroy_ob_stream, get_encoder, AVPixelFormat, AVPixelFormat_AV_PIX_FMT_BGRA, AVPixelFormat_AV_PIX_FMT_YUV420P, ObStream};

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

#[test]
fn test_ob_stream_new(){
    #[test]
    fn test_ob_stream_new(){
        let mut stream = ObStream::new(1920, 1080, AVPixelFormat_AV_PIX_FMT_BGRA, AVPixelFormat_AV_PIX_FMT_YUV420P);
        println!("Created stream: {:?}", stream);

        // 检查所有指针
        println!("ctx: {:?}", stream.ctx);
        println!("rb: {:?}", stream.rb);
        println!("tmp_buffer: {:?}", stream.tmp_buffer);
        println!("tmp_frame: {:?}", stream.tmp_frame);
        println!("dst_frame: {:?}", stream.dst_frame);

        stream.destroy();
    }

}