mod img;
mod nv12;
mod screen;

use crate::nv12::NV12Organizer;
use obcoder::stream::{ObEncoderVideo, ObStream};
use scap::capturer::{Capturer, Options};
use scap::frame::{Frame, FrameType};
use screen::get_screen_size;
use std::any::Any;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use trace_func::instrument;

struct ScreenCapture {
    capture: Capturer,
    width: f64,
    height: f64,
    bytes_per_row: u32,
    frame_type: FrameType,
}

impl ScreenCapture {
    fn new(width: f64, height: f64, format: &str) -> Self {
        let mut frame_type = FrameType::BGRAFrame;
        match format.to_uppercase().as_str() {
            "NV12" => frame_type = FrameType::YUVFrame,
            _ => {}
        }
        let options = Options {
            fps: 60,
            target: None, // None captures the primary display
            show_cursor: true,
            show_highlight: true,
            excluded_targets: None,
            output_type: frame_type,
            output_resolution: scap::capturer::Resolution::Captured,
            // crop_area: Some(Area {
            //     origin: Point { x: 0.0, y: 0.0 },
            //     size: Size {
            //         width,
            //         height,
            //     },
            // }),
            ..Default::default()
        };
        let mut capture = Capturer::build(options).expect("can not build Capturer");
        capture.start_capture();
        let cap = get_screen_size();

        Self {
            capture,
            width: cap.0 as f64,
            height: cap.1 as f64,
            bytes_per_row: cap.0 as u32 * 4,
            frame_type,
        }
    }

    pub fn get_desktop_capture_size(&self) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        // 获取第一帧来确定实际捕获尺寸
        let frame = self.capture.get_next_frame()?;
        match frame {
            Frame::BGRA(frame) => Ok((frame.width as f64, frame.height as f64)),
            Frame::YUVFrame(frame) => Ok((frame.width as f64, frame.height as f64)),
            _ => Err(Box::from("get captrue size failed!")),
        }
    }

    fn init(width: f64, height: f64, format: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !scap::is_supported() {
            println!("❌ Platform not supported");
            return Err(Box::from("Platform not supported!"));
        }

        if !scap::has_permission() {
            println!("❌ Permission not granted. Requesting permission...");
            if !scap::request_permission() {
                println!("❌ Permission denied");
                return Err(Box::from("Permission denied!"));
            }
        }

        Ok(Self::new(width, height, format))
    }

    fn get_capture(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let frame = self.capture.get_next_frame()?;

        match frame {
            Frame::BGRA(frame) => {
                let buffer = frame.data;
                let width = frame.width;
                let height = frame.height;
                self.width = width as f64;
                self.height = height as f64;
                self.bytes_per_row = (width * 4) as u32;
                Ok(buffer)
            }
            Frame::YUVFrame(frame) => {
                //   let data = NV12Organizer::organize_nv12_data(
                //     &frame.luminance_bytes,
                //     frame.luminance_stride as usize,
                //     &frame.chrominance_bytes,
                //     frame.chrominance_stride as usize,
                //     frame.width as usize,
                //     frame.height as usize,
                // );
                // match data {
                //     Ok(nv12) => Ok(nv12),
                //     Err(_) => Err("NV12 Organized Failed!".into()),
                // }
                let mut nv12_data = vec![0u8; (frame.width * frame.height) as usize * 3 / 2];
                unsafe {
                    NV12Organizer::organize_nv12_data_unchecked(
                        frame.luminance_bytes.as_ptr(),
                        frame.luminance_stride as usize,
                        frame.chrominance_bytes.as_ptr(),
                        frame.chrominance_stride as usize,
                        frame.width as usize,
                        frame.height as usize,
                        nv12_data.as_mut_ptr(),
                    )
                }
                Ok(nv12_data)
            }
            _ => Err(Box::from("can not match frame type!")),
        }
    }

    #[instrument]
    fn capture_frame(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let image = self.get_capture()?;
        Ok(image)
    }

    fn close(&mut self) {
        self.capture.stop_capture();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let format = "nv12";
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
    let ptr = Arc::from(Mutex::from(stream));
    let mut video = ObEncoderVideo::new(ptr.clone())?;

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

                if let Ok(ref mut mutex) = ptr.try_lock() {
                    mutex.write_frame(&frame_data)?;
                };
                let pkts = video.encode_available_frames()?;
                println!("send packets: {:?}", pkts.len());

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
