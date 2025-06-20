use ffmpeg_next::format;
use ffmpeg_next::format::Pixel;
use ringbuf::HeapRb;
use ringbuf::traits::*;
use ffmpeg_next as ff;

pub struct ObStream {
    ctx: ff::software::scaling::Context,
    pub(crate) width: u32,
    pub(crate) height: u32,
    src_pixel: format::Pixel,
    pub(crate) dst_pixel: format::Pixel,
    buffer: HeapRb<u8>,
    frame_size: usize,    // 单帧大小，用于优化
    temp_buffer: Vec<u8>, // 修改：移除了错误的 Box<&'static mut Vec<u8>>
}

impl ObStream {
    pub fn new(
        width: u32,
        height: u32,
        buffer_frames: usize, // 缓冲帧数
        src_format: &str,
    ) -> Result<ObStream, Box<dyn std::error::Error>> {
        let mut src_pixel = Pixel::None;
        match src_format.to_uppercase().as_str() {
            "NV12" => {
                src_pixel = Pixel::NV12;
            }
            _ => {
                src_pixel = Pixel::BGRA;
            }
        }
        let dst_pixel = format::Pixel::NV12;
        let ctx = ff::software::scaling::Context::get(
            src_pixel,
            width,
            height,
            dst_pixel,
            width,
            height,
            ffmpeg_next::software::scaling::flag::Flags::FAST_BILINEAR,
        )?;

        // 计算目标格式的帧大小
        let frame_size = Self::calculate_frame_size(src_pixel, width, height);
        let buffer_size = frame_size * buffer_frames;

        Ok(ObStream {
            ctx,
            width,
            height,
            src_pixel,
            dst_pixel,
            buffer: HeapRb::new(buffer_size), // 使用计算的缓冲区大小
            frame_size,
            temp_buffer: Vec::with_capacity(frame_size), // 修改：直接使用Vec
        })
    }

    pub(crate) fn calculate_frame_size(pixel: Pixel, width: u32, height: u32) -> usize {
        let pixels = (width * height) as usize;
        match pixel {
            format::Pixel::NV12 => pixels * 3 / 2,
            format::Pixel::YUV420P => pixels * 3 / 2,
            format::Pixel::BGRA | format::Pixel::RGBA => pixels * 4,
            format::Pixel::BGR24 | format::Pixel::RGB24 => pixels * 3,
            format::Pixel::YUV422P => pixels * 2,
            format::Pixel::YUV444P => pixels * 3,
            _ => pixels * 4, // 默认假设4字节
        }
    }

    pub fn write_slice(&mut self, data: &[u8]) -> usize {
        self.buffer.push_slice(data)
    }

    pub fn read_frame(&mut self) -> Result<Option<ff::frame::Video>, ff::util::error::Error> {
        if self.buffer.occupied_len() < self.frame_size {
            return Ok(None);
        }

        // 确保temp_buffer有足够空间
        self.temp_buffer.resize(self.frame_size, 0);
        let read_bytes = self.buffer.pop_slice(&mut self.temp_buffer);

        if read_bytes != self.frame_size {
            return Err(ff::util::error::Error::InvalidData);
        }

        match self.src_pixel {
            Pixel::NV12 => Ok(Option::from(ff::frame::Video::new(
                self.src_pixel,
                self.width,
                self.height,
            ))),
            Pixel::BGRA => {
                let mut dst_frame = ff::frame::Video::new(self.dst_pixel, self.width, self.height);
                let mut src_frame = ff::frame::Video::new(self.src_pixel, self.width, self.height);

                // 安全的数据拷贝
                let src_data = src_frame.data_mut(0);
                if src_data.len() < self.temp_buffer.len() {
                    return Err(ff::util::error::Error::InvalidData);
                }

                src_data[..self.temp_buffer.len()].copy_from_slice(&self.temp_buffer);

                match self.ctx.run(&src_frame, &mut dst_frame) {
                    Ok(_) => Ok(Some(dst_frame)),
                    Err(e) => Err(e),
                }
            }
            _ => Err(ff::Error::InvalidData),
        }
    }

    pub fn available_frames(&self) -> usize {
        self.buffer.occupied_len() / self.frame_size
    }

    pub fn buffer_info(&self) -> (usize, usize, usize) {
        (
            self.buffer.occupied_len(),
            self.buffer.vacant_len(),
            usize::from(self.buffer.capacity()),
        )
    }

    pub fn can_write_frame(&self) -> bool {
        self.buffer.vacant_len() >= self.frame_size
    }

    pub fn write_frame(&mut self, frame_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if frame_data.len() != self.frame_size {
            return Err(format!(
                "Frame data size mismatch: expected {}, got {}",
                self.frame_size,
                frame_data.len()
            )
                .into());
        }

        if !self.can_write_frame() {
            return Err("Buffer full".into());
        }

        let written = self.write_slice(frame_data);
        if written != frame_data.len() {
            Err("Partial write".into())
        } else {
            Ok(())
        }
    }
}