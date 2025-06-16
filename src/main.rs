
use wgpu::util::DeviceExt;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio::sync::oneshot;

struct ScreenCapture {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    texture: wgpu::Texture,
    buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    bytes_per_row: u32,
}

impl ScreenCapture {
    async fn new(width: u32, height: u32) -> Self {
        println!("初始化 GPU 设备，目标尺寸: {}x{}", width, height);

        // 创建 wgpu 实例
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 获取适配器
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // 创建设备和队列
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Screen Capture Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // 计算对齐的 bytes_per_row (必须是 256 的倍数)
        let unpadded_bytes_per_row = 4 * width;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let bytes_per_row = (unpadded_bytes_per_row + alignment - 1) / alignment * alignment;

        println!("bytes_per_row: {} (未对齐: {})", bytes_per_row, unpadded_bytes_per_row);

        // 创建纹理用于存储屏幕截图
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Screen Capture Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // 创建缓冲区用于读取纹理数据
        let buffer_size = (bytes_per_row * height) as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screen Capture Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,
            texture,
            buffer,
            width,
            height,
            bytes_per_row,
        }
    }

    async fn capture_frame(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 使用 screenshots crate 获取屏幕截图
        let screen = screenshots::Screen::all()?[0];
        let image = screen.capture()?;

        // 获取实际的图像尺寸
        let actual_width = image.width();
        let actual_height = image.height();

        println!("实际截图尺寸: {}x{}, 期望尺寸: {}x{}",
                 actual_width, actual_height, self.width, self.height);

        // 检查尺寸是否匹配
        if actual_width != self.width || actual_height != self.height {
            return Err(format!(
                "截图尺寸不匹配！实际: {}x{}, 期望: {}x{}",
                actual_width, actual_height, self.width, self.height
            ).into());
        }

        let image_data = image.rgba();

        // 直接使用原始图像数据，不进行额外的填充处理
        // 因为 write_texture 会自动处理对齐
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width), // 使用原始的 bytes_per_row
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        // 创建命令编码器
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Screen Capture Encoder"),
        });

        // 将纹理数据复制到缓冲区 - 这里需要使用对齐的 bytes_per_row
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.bytes_per_row), // 使用对齐的值
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        // 提交命令
        self.queue.submit(std::iter::once(encoder.finish()));

        // 映射缓冲区并读取数据
        let buffer_slice = self.buffer.slice(..);
        let (sender, receiver) = oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
            let _ = sender.send(v);
        });

        self.device.poll(wgpu::Maintain::Wait);
        receiver.await.unwrap()?;

        let data = buffer_slice.get_mapped_range();

        // 处理填充的数据，只提取实际的像素数据
        let mut result = Vec::with_capacity((self.width * self.height * 4) as usize);
        let unpadded_bytes_per_row = (self.width * 4) as usize;
        let padded_bytes_per_row = self.bytes_per_row as usize;

        for row in 0..self.height {
            let start = (row as usize) * padded_bytes_per_row;
            let end = start + unpadded_bytes_per_row;
            result.extend_from_slice(&data[start..end]);
        }

        drop(data);
        self.buffer.unmap();

        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始初始化屏幕捕获...");

    // 首先获取一张测试截图来确定实际尺寸
    let screens = screenshots::Screen::all()?;
    let screen = &screens[0];

    println!("显示器信息: {}x{}", screen.display_info.width, screen.display_info.height);

    // 获取一张测试截图来确定实际尺寸
    let test_image = screen.capture()?;
    let actual_width = test_image.width();
    let actual_height = test_image.height();

    println!("实际截图尺寸: {}x{}", actual_width, actual_height);

    // 使用实际的截图尺寸来创建捕获实例
    let capture = ScreenCapture::new(actual_width, actual_height).await;

    // 设置捕获帧率 (30 FPS)
    let mut interval = interval(Duration::from_millis(33));
    let mut frame_count = 0;

    println!("开始捕获视频流...");

    loop {
        interval.tick().await;

        match capture.capture_frame().await {
            Ok(frame_data) => {
                frame_count += 1;
                println!("捕获第 {} 帧，数据大小: {} 字节", frame_count, frame_data.len());

                // 示例：保存前10帧为PNG文件
                if frame_count <= 10 {
                    save_frame_as_png(&frame_data, actual_width, actual_height, frame_count)?;
                }

                // 演示用：捕获100帧后退出
                if frame_count >= 100 {
                    break;
                }
            }
            Err(e) => {
                eprintln!("捕获帧时出错: {}", e);
                break;
            }
        }
    }

    println!("视频流捕获完成！");
    Ok(())
}

fn save_frame_as_png(data: &[u8], width: u32, height: u32, frame_num: u32) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgba};

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, data.to_vec())
        .ok_or("无法创建图像缓冲区")?;

    let filename = format!("frame_{:03}.png", frame_num);
    img.save(&filename)?;
    println!("保存帧: {}", filename);

    Ok(())
}