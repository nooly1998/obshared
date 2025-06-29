//
// Created by 10904 on 25-6-27.
//

#ifndef OB_ENCODER_H
#define OB_ENCODER_H

#include <libavcodec/codec.h>

const char *encoders[] = {
#if defined(_WIN32) || defined(_WIN64)
    "h264_nvenc",
    "h264_amf",
    "h264_qsv",
#endif

#if defined(__APPLE__)
    "h264_videotoolbox",
#endif

#if defined(__linux__)
    "h264_nvenc",
    "h264_qsv",
    "h264_vaapi",
#endif
    NULL
};

const char *decoders[] = {
#if defined(_WIN32) || defined(_WIN64)
    "h264_nvdec",    // 1. 最佳 - 新NVIDIA硬件解码
    "h264_cuvid",    // 2. 次优 - 老NVIDIA硬件解码
    "h264_qsv",      // 3. 良好 - Intel硬件解码
    "h264_amf",      // 4. 良好 - AMD硬件解码
    "h264_d3d11va",  // 5. 中等 - 通用Windows硬件解码
    "h264_dxva2",    // 6. 较低 - 老Windows硬件解码接口
#endif

#if defined(__APPLE__)
    "h264_videotoolbox",
#endif

#if defined(__linux__)
    "h264_nvdec",    // NVIDIA 最新
    "h264_cuvid",    // NVIDIA CUDA
    "h264_vdpau",    // NVIDIA VDPAU (Linux传统)
    "h264_qsv",      // Intel
    "h264_vaapi",    // Linux 通用 (Intel/AMD)
#endif
    NULL
};

const AVCodec * get_encoder();
const AVCodec * get_decoder();
void list_available_encoders();

#endif //OB_ENCODER_H
