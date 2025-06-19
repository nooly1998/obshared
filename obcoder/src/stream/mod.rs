use crate::coder;
use ffmpeg_next as ff;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::{format, Codec};
use ringbuf::traits::*;
use ringbuf::HeapRb;
use std::sync::{Arc, Mutex};

pub struct ObStream {
    ctx: ff::software::scaling::Context,
    width: u32,
    height: u32,
    src_pixel: format::Pixel,
    dst_pixel: format::Pixel,
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

    fn calculate_frame_size(pixel: Pixel, width: u32, height: u32) -> usize {
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

pub struct ObEncoderVideo {
    stream: Arc<Mutex<ObStream>>, // 修改：使用 Arc<Mutex<ObStream>>
    codec: Codec,
    encoder: ff::codec::encoder::Video,
    packet_buffer: Vec<ff::packet::Packet>,
}

impl ObEncoderVideo {
    pub fn new(stream: Arc<Mutex<ObStream>>) -> Result<Self, Box<dyn std::error::Error>> {
        let codec = coder::encoder()?;
        let mut context = ff::codec::Context::new_with_codec(codec);

        // 获取流的参数
        let (width, height, dst_pixel) = {
            let stream_guard = stream.lock().unwrap();
            (
                stream_guard.width,
                stream_guard.height,
                stream_guard.dst_pixel,
            )
        };

        // 配置编码器参数
        let mut encoder = context.encoder().video()?;
        encoder.set_width(width);
        encoder.set_height(height);
        encoder.set_format(dst_pixel);
        encoder.set_bit_rate(1000000); // 1 Mbps
        encoder.set_max_bit_rate(2000000);
        encoder.set_time_base(ff::util::rational::Rational(1, 30)); // 30 FPS
        encoder.set_gop(30); // GOP size

        // 打开编码器
        let encoder = encoder.open_as(codec)?;

        Ok(ObEncoderVideo {
            stream,
            codec,
            encoder,
            packet_buffer: vec![],
        })
    }

    pub fn encode_available_frames(&mut self) -> Result<Vec<ff::packet::Packet>, ff::Error> {
        let mut packets = Vec::new();

        // 获取可用帧数（不需要持有锁太久）
        let available_frames = {
            let stream = self.stream.lock().unwrap();
            stream.available_frames()
        };

        if available_frames == 0 {
            return Ok(packets);
        }

        // 预分配容量
        packets.reserve(available_frames);

        // 批处理帧
        const BATCH_SIZE: usize = 10;
        let mut processed_frames = 0;

        loop {
            // 每次循环都获取新的锁
            let frame_result = {
                let mut stream = self.stream.lock().unwrap();
                stream.read_frame()
            };

            match frame_result {
                Ok(Some(frame)) => {
                    self.encoder.send_frame(&frame)?;
                    processed_frames += 1;

                    // 每处理一批帧后收集数据包
                    if processed_frames % BATCH_SIZE == 0 {
                        self.collect_packets(&mut packets)?;
                    }
                }
                Ok(None) => break, // 没有更多帧
                Err(e) => return Err(e.into()),
            }
        }

        // 处理剩余的数据包
        self.collect_packets(&mut packets)?;

        Ok(packets)
    }

    pub fn flush(&mut self) -> Result<Vec<ff::packet::Packet>, ff::Error> {
        let mut packets = Vec::new();

        // 发送空帧表示编码结束
        self.encoder.send_frame(&ff::frame::Video::empty())?;

        // 收集所有剩余数据包
        loop {
            let mut packet = ff::packet::Packet::empty();
            match self.encoder.receive_packet(&mut packet) {
                Ok(()) => packets.push(packet),
                Err(ff::Error::Eof) => break,
                Err(ff::Error::Other { errno }) if errno == ff::util::error::EAGAIN => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(packets)
    }

    fn collect_packets(&mut self, packets: &mut Vec<ff::packet::Packet>) -> Result<(), ff::Error> {
        loop {
            let mut packet = ff::packet::Packet::empty();
            match self.encoder.receive_packet(&mut packet) {
                Ok(()) => packets.push(packet),
                Err(ff::Error::Other { errno }) if errno == ff::util::error::EAGAIN => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

// 使用示例和测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_size_calculation() {
        assert_eq!(
            ObStream::calculate_frame_size(format::Pixel::YUV420P, 1920, 1080),
            1920 * 1080 * 3 / 2
        );
        assert_eq!(
            ObStream::calculate_frame_size(format::Pixel::RGBA, 1920, 1080),
            1920 * 1080 * 4
        );
    }
}
