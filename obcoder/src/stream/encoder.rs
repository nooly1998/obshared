use std::sync::{Arc, Mutex};
use ffmpeg_next::Codec;
use crate::stream::obstream::ObStream;
use crate::coder::codec as coder;
use ffmpeg_next as ff;
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