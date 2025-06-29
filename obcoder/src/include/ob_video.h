//
// Created by 10904 on 25-6-27.
//

#ifndef OB_VIDEO_H
#define OB_VIDEO_H
#include <libavcodec/codec.h>
#include <libavcodec/avcodec.h>
#include "ob_stream.h"

typedef struct {
    ObStream* stream;
    const AVCodec* codec;
    AVCodecContext* ctx;
    AVCodecParameters* params;
    enum AVPixelFormat pixel_format;
    int width;
    int height;
}ObEncoder;

ObEncoder* ob_encoder_new();
int ob_encoder_init(ObEncoder* encoder, const AVCodec* codec, ObStream* stream, int width, int height);
int ob_encoder_open(ObEncoder* encoder);
int ob_encoder_encode(ObEncoder* encoder, AVPacket* recv_packet);
void ob_encoder_close(ObEncoder* encoder);
void ob_encoder_destroy(ObEncoder* encoder);

#endif //OB_VIDEO_H
