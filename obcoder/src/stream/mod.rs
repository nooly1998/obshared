pub mod obstream;
pub mod encoder;


use ffmpeg_next::{format, Codec};



use crate::stream::obstream::ObStream;



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
