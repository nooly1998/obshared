use ffmpeg_next as ff;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::sys::AVFrame;

pub(crate) fn from_bgra_to_yuv420p(data: &[u8], width: u32, height: u32, mut ctx: &ff::software::scaling::Context) -> Result<AVFrame, dyn std::error::Error> {
    let mut input_frame = ff::util::frame::Video::new(Pixel::BGRA, width, height);
    input_frame.data_mut(0).copy_from_slice(data);
    let mut output_frame = ff::util::frame::Video::new(Pixel::YUV420P, width, height);
    ctx.run(&input_frame, &mut output_frame)?;
    Ok(output_frame.into())
}