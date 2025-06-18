use yuv::*;

pub fn save_frame_as_png(
    data: &[u8],
    width: u32,
    height: u32,
    frame_num: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgba};

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, data.to_vec()).ok_or("无法创建图像缓冲区")?;

    let filename = format!("frame_{:03}.png", frame_num);
    img.save(&filename)?;
    println!("保存帧: {}", filename);

    Ok(())
}