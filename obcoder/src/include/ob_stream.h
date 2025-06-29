#ifndef OB_STREAM_H
#define OB_STREAM_H
#include "libswscale/swscale.h"
#include "ring_buffer.h"
#include "libavutil/imgutils.h"


typedef struct {
    /// 转换Swscale 上下文
    struct SwsContext* ctx;
    /// 图像宽
    int width;
    /// 图像高
    int height;
    /// 源 格式
    enum AVPixelFormat src_pixel;
    /// 目标 格式
    enum AVPixelFormat dst_pixel;
    /// 环形缓冲区
    ring_buffer_t* rb;
    /// 缓冲区预设帧大小
    int frame_count;
    /// 输出缓冲区
    uint8_t* tmp_buffer;
    /// 输出缓冲区大小
    size_t tmp_buffer_size;
    /// 源 缓存帧
    AVFrame* tmp_frame;
    /// 目标 缓存帧
    AVFrame* dst_frame;
} ObStream;

/// 创建转换流
/// @param width 图像宽
/// @param height 图像高
/// @param frame_count 设置缓存大小
/// @param src_pixel 设置来源色彩格式
/// @param dst_pixel 设置目标色彩格式
/// @return 转换流指针
ObStream* create_ob_stream(int width, int height, int frame_count,enum AVPixelFormat src_pixel, enum AVPixelFormat dst_pixel);

///
/// @param ob_stream 释放转换流内存
void destroy_ob_stream(ObStream* ob_stream);

///
/// @param ob_stream 向转换流中写入帧
/// @param data 写入数据
/// @param size 写入数据大小
/// @return err
int ob_stream_write_frame(ObStream* ob_stream, uint8_t* data,size_t size);

/// 获取目标帧，在此方法中调用 swscale 输出为对内部缓存目标帧的引用，需使用 av_frame_unref 释放引用
/// @param ob_stream 转换流
/// @param dst_frame 引用内部缓存的帧，需要在外部调用 av_frame_unref
/// @return err
int  ob_stream_get_frame(ObStream* ob_stream,AVFrame* dst_frame);

///
/// @param width 图像宽
/// @param height 图像高
/// @return 图像预计大小
size_t calculate_frame_size(int width, int height, enum AVPixelFormat);

#endif //OB_STREAM_H