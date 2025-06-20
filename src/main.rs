mod img;
mod nv12;
mod screen;
mod capture;

use crate::nv12::NV12Organizer;
use scap::capturer::{Area, Capturer, Options, Point, Size};
use scap::frame::{Frame, FrameType};
use screen::get_screen_size;
use std::any::Any;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use obcoder::stream::encoder::ObEncoderVideo;
use obcoder::stream::obstream::ObStream;
use trace_func::instrument;
use crate::capture::screencap::ScreenCapture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let format = "bgra";
    // 使用实际的截图尺寸来创建捕获实例
    let mut capture = ScreenCapture::init(0.0, 0.0, format)?;

    println!("开始初始化屏幕捕获...");
    let (actual_width, actual_height) = capture.get_desktop_capture_size()?;
    println!("显示器信息: {}x{}", actual_width, actual_height);

    println!("实际截图尺寸: {}x{}", actual_width, actual_height);

    // 设置捕获帧率 (30 FPS)
    let mut frame_count = 0;

    println!("开始捕获视频流...");
    let stream = ObStream::new(actual_width as u32, actual_height as u32, 2, format)?;
    // let ptr = Arc::from(Mutex::from(stream));
    // let mut video = ObEncoderVideo::new(ptr.clone())?;

    loop {
        match capture.capture_frame() {
            Ok(mut frame_data) => {
                frame_count += 1;
                println!(
                    "捕获第 {} 帧，{} x {}, 数据大小: {} 字节",
                    frame_count,
                    actual_width,
                    actual_height,
                    frame_data.len()
                );
                if frame_data.len() == 0 {
                    continue;
                }

                // if let Ok(ref mut mutex) = ptr.try_lock() {
                //     mutex.write_frame(&frame_data)?;
                // };
                // let pkts = video.encode_available_frames()?;
                // println!("send packets: {:?}", pkts.len());

                // 演示用：捕获100帧后退出
                if frame_count >= 100 {
                    break;
                }
            }
            Err(e) => {
                eprintln!("捕获帧时出错: {}", e);
                break;
            }
        }
    }
    capture.close();
    // let last_pkt_vec = video.flush()?;
    // println!("last packet vector: {:?}", last_pkt_vec.len());
    println!("视频流捕获完成！");
    Ok(())
}
