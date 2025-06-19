use image::{ImageBuffer, Rgb};


pub fn save_frame_as_png(
    data: &[u8],
    width: u32,
    height: u32,
    frame_num: u32,
) -> Result<(), Box<dyn std::error::Error>> {

    let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, data.to_vec()).ok_or("Cannot create image buffer")?;

    let filename = format!("frame_{:03}.png", frame_num);
    img.save(&filename)?;
    println!("Saved frame: {}", filename);

    Ok(())
}

pub fn get_yuv_corrected(
    lu_data: &Vec<u8>,    // 完整的Y平面数据
    lu_stride: i32,     // Y平面的行跨度 (bytes per row)
    ch_data: &Vec<u8>,    // 包含U平面后紧跟V平面的数据
    ch_stride: i32,     // U和V平面各自的行跨度
    width: i32,         // 图像宽度
    height: i32,        // 图像高度
) -> Vec<u8> {
    let width_u = width as usize;
    let height_u = height as usize;
    let lu_stride_u = lu_stride as usize;
    let ch_stride_u = ch_stride as usize;

    let ch_width_u = (width / 2) as usize;
    let ch_height_u = (height / 2) as usize;

    // 预估总大小并预分配内存
    let y_plane_size = width_u * height_u; // 实际拷贝的Y数据大小
    let uv_plane_size_each = ch_width_u * ch_height_u; // 实际拷贝的U或V数据大小
    let mut result = Vec::with_capacity(y_plane_size + 2 * uv_plane_size_each);

    // 1. 复制 Y 平面数据
    for r in 0..height_u {
        let row_start = r * lu_stride_u;
        let row_end = row_start + width_u;
        if row_end <= lu_data.len() {
            result.extend_from_slice(&lu_data[row_start..row_end]);
        } else {
            // 错误处理：数据不足
            eprintln!("Error: Not enough luminance data for row {}", r);
            return Vec::new(); // 或其他错误处理
        }
    }

    // 2. 复制 U 平面数据
    // 假设 U 平面数据在 ch_data 的开头
    for r in 0..ch_height_u {
        let row_start = r * ch_stride_u;
        let row_end = row_start + ch_width_u;
        if row_end <= ch_data.len() { // 确保不越界整个 ch_data
            result.extend_from_slice(&ch_data[row_start..row_end]);
        } else {
            eprintln!("Error: Not enough U-plane data for row {}", r);
            return Vec::new();
        }
    }

    // 3. 复制 V 平面数据
    // V 平面数据紧跟在 U 平面数据之后
    // U平面占用的总字节数（包括可能的 stride 填充）
    let u_plane_total_bytes_in_ch = ch_height_u * ch_stride_u;

    for r in 0..ch_height_u {
        let v_row_start_in_ch_block = u_plane_total_bytes_in_ch + (r * ch_stride_u);
        let v_row_end_in_ch_block = v_row_start_in_ch_block + ch_width_u;
        if v_row_end_in_ch_block <= ch_data.len() {
            result.extend_from_slice(&ch_data[v_row_start_in_ch_block..v_row_end_in_ch_block]);
        } else {
            eprintln!("Error: Not enough V-plane data for row {}", r);
            return Vec::new();
        }
    }

    result
}



