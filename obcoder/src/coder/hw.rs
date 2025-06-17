///
/// return the current platform's accel hw encoder str sgn,
/// used for ffmpeg find_by_str
///
pub(crate) fn get_hw_accel_encoder() -> Vec<&'static str> {
    let mut hws = Vec::new();

    #[cfg(target_os = "macos")]
    hws.extend_from_slice(&["h264_videotoolbox"]);

    #[cfg(target_os = "windows")]
    hws.extend_from_slice(&["h264_nvenc", "h264_amf", "h264_qsv"]);

    #[cfg(target_os = "linux")]
    hws.extend_from_slice(&["h264_nvenc", "h264_qsv", "h264_vaapi"]);

    // 添加软件编码器作为后备选项
    hws.push("libx264");

    hws
}

///
/// return the current platform's accel hw decoder str sgn,
/// used for ffmpeg find_by_str
///
pub(crate) fn get_hw_accel_decoder() -> Vec<&'static str> {
    let mut hws = Vec::new();

    #[cfg(target_os = "macos")]
    hws.extend_from_slice(&["h264_videotoolbox"]);

    #[cfg(target_os = "windows")]
    hws.extend_from_slice(&[
        "h264_nvdec",    // 首选新NVIDIA
        "h264_cuvid",    // 兼容老NVIDIA
        "h264_d3d11va",  // 通用Windows
        "h264_dxva2",    // 老Windows接口
        "h264_qsv",      // Intel
        "h264_amf",      // AMD
    ]);

    #[cfg(target_os = "linux")]
    hws.extend_from_slice(&[
        "h264_nvdec",    // 优先使用较新的 NVDEC
        "h264_cuvid",    // 后备选项
        "h264_vaapi", 
        "h264_vdpau"
    ]);

    // 添加软件解码器作为后备选项
    hws.push("h264");

    hws
}