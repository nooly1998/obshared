use crate::coder::hw;
use ffmpeg_next as ff;
/// 获取当前平台支持的硬件加速编码器
///
/// 该函数会根据当前操作系统返回最适合的编码器，
/// 优先选择硬件加速编码器，软件编码器作为后备选项
///
/// # Returns
///
/// 返回找到的第一个可用编码器，如果所有编码器都不可用则返回错误
///
/// # Errors
///
/// 当没有找到任何可用的编码器时返回错误
pub fn encoder() -> Result<ff::Codec,Box<dyn std::error::Error>>{
    let hws = hw::get_hw_accel_encoder();
    for hw in hws{
        if let Some(encoder) = ff::encoder::find_by_name(hw){
            println!("Codec：{:?} used",hw);
            return Ok(encoder);
        };
    }
    Err(Box::from("can not found Encoder Codec!"))
}

/// 获取当前平台支持的硬件加速解码器
///
/// 该函数会根据当前操作系统返回最适合的解码器，
/// 优先选择硬件加速解码器，软件解码器作为后备选项
///
/// # Returns
///
/// 返回找到的第一个可用解码器，如果所有解码器都不可用则返回错误
///
/// # Errors
///
/// 当没有找到任何可用的解码器时返回错误
pub fn decoder() -> Result<Box<ff::Codec>,Box<dyn std::error::Error>>{
    let hws = hw::get_hw_accel_decoder();
    for hw in hws{
        if let Some(decoder) = ff::decoder::find_by_name(hw){
            println!("Codec：{:?} used",hw);
            return Ok(Box::from(decoder));
        };
    }
    Err(Box::from("can not found Decoder Codec!"))
}