//
// Created by 10904 on 25-6-21.
//

#ifndef RING_BUFFER_H
#define RING_BUFFER_H

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

// 环形缓冲区结构体定义
typedef struct {
    char *buffer;       // 缓冲区数据存储
    size_t head;        // 头指针（写入位置）
    size_t tail;        // 尾指针（读取位置）
    size_t size;        // 缓冲区总大小
    bool full;          // 缓冲区是否已满
} ring_buffer_t;

ring_buffer_t* ring_buffer_create(size_t size);
void ring_buffer_destroy(ring_buffer_t *rb);
bool ring_buffer_empty(ring_buffer_t *rb);
bool ring_buffer_full(ring_buffer_t *rb);
size_t ring_buffer_size(ring_buffer_t *rb);
bool ring_buffer_put(ring_buffer_t *rb, char data);
bool ring_buffer_get(ring_buffer_t *rb, char *data);
void ring_buffer_reset(ring_buffer_t *rb);


#endif //RING_BUFFER_H
