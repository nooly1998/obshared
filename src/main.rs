
mod img;
mod screen;

use scap::capturer::{Capturer, Options};
use scap::frame::Frame;
use std::any::Any;
use tokio::time::{interval, Duration};

use screen::get_screen_size;


struct ScreenCapture {
    capture: Capturer,
    width: f64,
    height: f64,
    bytes_per_row: u32,
}

impl ScreenCapture {

    async fn new(width: f64, height: f64) -> Self {
        let options = Options {
            fps: 60,
            target: None, // None captures the primary display
            show_cursor: true,
            show_highlight: true,
            excluded_targets: None,
            output_type: scap::frame::FrameType::BGRAFrame,
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
            width:cap.0 as f64,
            height:cap.1 as f64,
            bytes_per_row: cap.0 as u32 * 4,
        }
    }

    pub fn get_desktop_capture_size(&self) -> Result<(f64, f64), Box<dyn std::error::Error>> {

        // 获取第一帧来确定实际捕获尺寸
        let frame = self.capture.get_next_frame()?;
        match frame {
            Frame::BGRA(frame) =>{
                Ok((frame.width as f64, frame.height as f64))
            },
            _ => {
                Err(Box::from("get captrue size failed!"))
            }
        }

    }


    async fn init(width: f64, height: f64) -> Result<Self, Box<dyn std::error::Error>> {
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

        Ok(Self::new(width, height).await)
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
            _ => Err(Box::from("can not match frame type!")),
        }
    }


    async fn capture_frame(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let image = self.get_capture()?;
        Ok(image)
    }

    async fn close(&mut self) {
        self.capture.stop_capture();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用实际的截图尺寸来创建捕获实例
    let mut capture = ScreenCapture::init(0.0,0.0).await?;

    println!("开始初始化屏幕捕获...");
    let (actual_width,actual_height) = capture.get_desktop_capture_size()?;
    println!(
        "显示器信息: {}x{}",
        actual_width, actual_height
    );

    println!("实际截图尺寸: {}x{}", actual_width, actual_height);


    // 设置捕获帧率 (30 FPS)
    let mut interval = interval(Duration::from_millis(33));
    let mut frame_count = 0;

    println!("开始捕获视频流...");

    loop {
        interval.tick().await;

        match capture.capture_frame().await {
            Ok(mut frame_data) => {
                frame_count += 1;
                println!(
                    "捕获第 {} 帧，数据大小: {} 字节",
                    frame_count,
                    frame_data.len()
                );

                if frame_count <= 10 {
                    //TODO: to stream
                }

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
    capture.close().await;
    println!("视频流捕获完成！");
    Ok(())
}