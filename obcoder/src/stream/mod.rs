use std::io::Bytes;
use crate::coder;
use ffmpeg_next as ff;
mod convert;

fn encoder_video(data:&[u8],width:u32,height:u32,){
    //TODO: 避免ff::software::scaling::Context 创建销毁开销，必须从外部传入，\\\
    // 初始化参数包含src pixel format, dst pixel format, 因此pixel也必须从外部指定, \\\
    // 目前方法from_bgra_to_yuv420p在内部指定pixel format，需要重新考虑

    // let encoder = coder::encoder();
    // let ctx = ff::software::scaling::Context::get()
    // convert::from_bgra_to_yuv420p(data,width,height);
}
