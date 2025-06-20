use crate::nv12::NV12Organizer;
use crate::screen::get_screen_size;
use scap::capturer::{Capturer, Options};
use scap::frame::{Frame, FrameType};
use trace_func::instrument;

pub(crate) struct ScreenCapture {
    capture: Capturer,
    width: f64,
    height: f64,
    bytes_per_row: u32,
    nv12buffer: Vec<u8>,
    bgra_buffer: Vec<u8>,
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
            fps: 120,
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
            bgra_buffer: vec![0u8; 20_736_000],
            nv12buffer: vec![0u8; 7_776_000],
            frame_type,
        }
    }

    pub(crate) fn get_desktop_capture_size(
        &self,
    ) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        // 获取第一帧来确定实际捕获尺寸
        let frame = self.capture.get_next_frame()?;
        match frame {
            Frame::BGRA(frame) => Ok((frame.width as f64, frame.height as f64)),
            Frame::YUVFrame(frame) => Ok((frame.width as f64, frame.height as f64)),
            _ => Err(Box::from("get captrue size failed!")),
        }
    }

    pub(crate) fn init(
        width: f64,
        height: f64,
        format: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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

    fn get_capture(&mut self) -> Result<&Vec<u8>, Box<dyn std::error::Error>> {
        let frame = self.capture.get_next_frame()?;

        match frame {
            Frame::BGRA(frame) => {
                if frame.data.len() == 0 {
                    return Err("frame data is null！".into())
                }
                self.bgra_buffer.copy_from_slice(&frame.data);
                let width = frame.width;
                let height = frame.height;
                self.width = width as f64;
                self.height = height as f64;
                self.bytes_per_row = (width * 4) as u32;
                Ok(&self.bgra_buffer)
            }
            Frame::YUVFrame(frame) => {
                // let mut nv12_data = vec![0u8; (frame.width * frame.height) as usize * 3 / 2];
                unsafe {
                    NV12Organizer::organize_nv12_data_unchecked(
                        frame.luminance_bytes.as_ptr(),
                        frame.luminance_stride as usize,
                        frame.chrominance_bytes.as_ptr(),
                        frame.chrominance_stride as usize,
                        frame.width as usize,
                        frame.height as usize,
                        self.nv12buffer.as_mut_ptr(),
                    )
                }
                Ok(&self.nv12buffer)
            }
            _ => Err(Box::from("can not match frame type!")),
        }
    }

    #[instrument]
    pub(crate) fn capture_frame(&mut self) -> Result<&Vec<u8>, Box<dyn std::error::Error>> {
        let image = self.get_capture()?;
        Ok(image)
    }

    pub(crate) fn close(&mut self) {
        self.capture.stop_capture();
    }
}
