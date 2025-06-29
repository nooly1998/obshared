mod bindings;

use std::error::Error;
pub use bindings::*;
use std::ffi::{CStr, CString};

mod test;

pub fn calc_image_size(width:i32,height:i32,format:AVPixelFormat)  -> usize {
    unsafe { return calculate_frame_size(width, height, format) };
}


impl ObStream {
    pub fn new(width:i32,height:i32,src_format:AVPixelFormat, dst_format:AVPixelFormat) -> Self {
        unsafe {  *create_ob_stream(width, height, 5, src_format,dst_format ) }
    }

    pub fn destroy(&mut self) {
        unsafe {
            destroy_ob_stream(self as *mut ObStream);
        }

    }

    pub fn write_frame(&mut self, image_data:&mut [u8], image_size:usize) ->Result<(),Box<dyn Error>>{
        let ret = unsafe {ob_stream_write_frame(self,image_data.as_mut_ptr(),image_size)};
        if ret == 0 {
            Ok(())
        }else{
            Err("can not write frame data!".into())
        }
    }

    ///
    /// you need to unref and free the AVFrame struct without rustc
    /// you get this `AVFrame` is ref a cache frame in ObStream,
    /// you must call `av_frame_unref` to unref the frame and use
    /// `av_frame_free` to free this struct's memory
    ///
    pub fn read_frame(&mut self) ->Result<AVFrame,Box<dyn Error>>{
        unsafe {
            let frame = av_frame_alloc();
            let ret = ob_stream_get_frame(self,frame);
            if ret == 0 {
                return Ok(*frame);
            }
            else {
                return Err("can not get frame data!".into());
            }
        }
    }
}
