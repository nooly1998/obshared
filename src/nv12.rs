use std::ptr;

/// NV12数据组织结构
pub struct NV12Organizer;

impl NV12Organizer {
    /// 将带stride的YUV数据组织为NV12格式
    pub fn organize_nv12_data(
        luminance_bytes: &[u8],
        luminance_stride: usize,
        chrominance_bytes: &[u8],
        chrominance_stride: usize,
        width: usize,
        height: usize,
    ) -> Result<Vec<u8>, NV12Error> {
        // 参数验证
        Self::validate_parameters(
            luminance_bytes,
            luminance_stride,
            chrominance_bytes,
            chrominance_stride,
            width,
            height,
        )?;

        let nv12_size = Self::calculate_nv12_size(width, height);
        let mut nv12_output = vec![0u8; nv12_size];

        Self::copy_luminance_plane(
            luminance_bytes,
            luminance_stride,
            &mut nv12_output,
            width,
            height,
        )?;

        Self::copy_chrominance_plane(
            chrominance_bytes,
            chrominance_stride,
            &mut nv12_output,
            width,
            height,
        )?;

        Ok(nv12_output)
    }

    /// 计算NV12数据总大小
    pub fn calculate_nv12_size(width: usize, height: usize) -> usize {
        width * height * 3 / 2
    }

    /// 参数验证
    fn validate_parameters(
        luminance_bytes: &[u8],
        luminance_stride: usize,
        chrominance_bytes: &[u8],
        chrominance_stride: usize,
        width: usize,
        height: usize,
    ) -> Result<(), NV12Error> {
        if luminance_stride < width {
            return Err(NV12Error::InvalidStride("luminance_stride < width".to_string()));
        }

        if chrominance_stride < width {
            return Err(NV12Error::InvalidStride("chrominance_stride < width".to_string()));
        }

        let required_y_size = height * luminance_stride;
        if luminance_bytes.len() < required_y_size {
            return Err(NV12Error::InsufficientData("luminance buffer too small".to_string()));
        }

        let uv_height = height / 2;
        let required_uv_size = uv_height * chrominance_stride;
        if chrominance_bytes.len() < required_uv_size {
            return Err(NV12Error::InsufficientData("chrominance buffer too small".to_string()));
        }

        Ok(())
    }

    /// 复制亮度平面
    fn copy_luminance_plane(
        luminance_bytes: &[u8],
        luminance_stride: usize,
        nv12_output: &mut [u8],
        width: usize,
        height: usize,
    ) -> Result<(), NV12Error> {
        let mut output_offset = 0;

        for y in 0..height {
            let src_offset = y * luminance_stride;
            let src_end = src_offset + width;

            if src_end > luminance_bytes.len() {
                return Err(NV12Error::IndexOutOfBounds("luminance data".to_string()));
            }

            let output_end = output_offset + width;
            if output_end > nv12_output.len() {
                return Err(NV12Error::IndexOutOfBounds("output buffer".to_string()));
            }

            // 只复制有效的像素数据，忽略padding
            nv12_output[output_offset..output_end]
                .copy_from_slice(&luminance_bytes[src_offset..src_end]);

            output_offset += width;
        }

        Ok(())
    }

    /// 复制色度平面
    fn copy_chrominance_plane(
        chrominance_bytes: &[u8],
        chrominance_stride: usize,
        nv12_output: &mut [u8],
        width: usize,
        height: usize,
    ) -> Result<(), NV12Error> {
        let y_plane_size = width * height;
        let mut output_offset = y_plane_size;
        let uv_height = height / 2;

        for y in 0..uv_height {
            let src_offset = y * chrominance_stride;
            let src_end = src_offset + width;

            if src_end > chrominance_bytes.len() {
                return Err(NV12Error::IndexOutOfBounds("chrominance data".to_string()));
            }

            let output_end = output_offset + width;
            if output_end > nv12_output.len() {
                return Err(NV12Error::IndexOutOfBounds("output buffer".to_string()));
            }

            // 只复制有效的UV数据，忽略padding
            nv12_output[output_offset..output_end]
                .copy_from_slice(&chrominance_bytes[src_offset..src_end]);

            output_offset += width;
        }

        Ok(())
    }

    /// 获取NV12数据中Y和UV平面的切片
    pub fn get_nv12_planes(nv12_data: &[u8], width: usize, height: usize) -> (&[u8], &[u8]) {
        let y_plane_size = width * height;
        let y_plane = &nv12_data[0..y_plane_size];
        let uv_plane = &nv12_data[y_plane_size..];
        (y_plane, uv_plane)
    }

    /// 获取NV12数据中Y和UV平面的可变切片
    pub fn get_nv12_planes_mut(nv12_data: &mut [u8], width: usize, height: usize) -> (&mut [u8], &mut [u8]) {
        let y_plane_size = width * height;
        let (y_plane, uv_plane) = nv12_data.split_at_mut(y_plane_size);
        (y_plane, uv_plane)
    }
}

/// 零拷贝版本（使用不安全代码，性能更好）
impl NV12Organizer {
    /// 零拷贝版本 - 直接操作指针
    pub unsafe fn organize_nv12_data_unchecked(
        luminance_bytes: *const u8,
        luminance_stride: usize,
        chrominance_bytes: *const u8,
        chrominance_stride: usize,
        width: usize,
        height: usize,
        nv12_output: *mut u8,
    ) {
        let mut output_ptr = nv12_output;

        // 复制Y平面
        for y in 0..height {
            let src_line = luminance_bytes.add(y * luminance_stride);
            ptr::copy_nonoverlapping(src_line, output_ptr, width);
            output_ptr = output_ptr.add(width);
        }

        // 复制UV平面
        let uv_height = height / 2;
        for y in 0..uv_height {
            let src_line = chrominance_bytes.add(y * chrominance_stride);
            ptr::copy_nonoverlapping(src_line, output_ptr, width);
            output_ptr = output_ptr.add(width);
        }
    }
}

/// 错误类型定义
#[derive(Debug, Clone)]
pub enum NV12Error {
    InvalidStride(String),
    InsufficientData(String),
    IndexOutOfBounds(String),
}

impl std::fmt::Display for NV12Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NV12Error::InvalidStride(msg) => write!(f, "Invalid stride: {}", msg),
            NV12Error::InsufficientData(msg) => write!(f, "Insufficient data: {}", msg),
            NV12Error::IndexOutOfBounds(msg) => write!(f, "Index out of bounds: {}", msg),
        }
    }
}

impl std::error::Error for NV12Error {}

/// 使用示例和测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nv12_organization() {
        let width = 4;
        let height = 4;
        let luminance_stride = 8; // 带padding
        let chrominance_stride = 8; // 带padding

        // 创建测试数据
        let mut luminance_data = vec![0u8; height * luminance_stride];
        let mut chrominance_data = vec![0u8; (height / 2) * chrominance_stride];

        // 填充Y数据（只填充有效像素）
        for y in 0..height {
            for x in 0..width {
                luminance_data[y * luminance_stride + x] = (y * width + x) as u8;
            }
        }

        // 填充UV数据（只填充有效像素）
        for y in 0..(height / 2) {
            for x in 0..width {
                chrominance_data[y * chrominance_stride + x] = ((y * width + x) + 100) as u8;
            }
        }

        // 组织为NV12
        let nv12_result = NV12Organizer::organize_nv12_data(
            &luminance_data,
            luminance_stride,
            &chrominance_data,
            chrominance_stride,
            width,
            height,
        );

        assert!(nv12_result.is_ok());
        let nv12_data = nv12_result.unwrap();

        // 验证大小
        assert_eq!(nv12_data.len(), width * height * 3 / 2);

        // 验证Y平面数据
        let (y_plane, uv_plane) = NV12Organizer::get_nv12_planes(&nv12_data, width, height);
        assert_eq!(y_plane.len(), width * height);
        assert_eq!(uv_plane.len(), width * height / 2);

        // 验证数据正确性
        for y in 0..height {
            for x in 0..width {
                let expected = (y * width + x) as u8;
                let actual = y_plane[y * width + x];
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn test_parameter_validation() {
        let width = 4;
        let height = 4;
        let luminance_stride = 2; // 错误：小于width
        let chrominance_stride = 4;

        let luminance_data = vec![0u8; height * luminance_stride];
        let chrominance_data = vec![0u8; (height / 2) * chrominance_stride];

        let result = NV12Organizer::organize_nv12_data(
            &luminance_data,
            luminance_stride,
            &chrominance_data,
            chrominance_stride,
            width,
            height,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            NV12Error::InvalidStride(_) => {},
            _ => panic!("Expected InvalidStride error"),
        }
    }
}

/// 实际使用示例
pub fn example_usage() {
    // 假设你有这些数据
    let width = 1920;
    let height = 1080;
    let luminance_stride = 1920; // 可能有padding
    let chrominance_stride = 1920; // 可能有padding

    // 假设的原始数据
    let luminance_data = vec![0u8; height * luminance_stride];
    let chrominance_data = vec![0u8; (height / 2) * chrominance_stride];

    // 组织为NV12
    match NV12Organizer::organize_nv12_data(
        &luminance_data,
        luminance_stride,
        &chrominance_data,
        chrominance_stride,
        width,
        height,
    ) {
        Ok(nv12_data) => {
            println!("NV12 data size: {} bytes", nv12_data.len());

            // 获取Y和UV平面
            let (y_plane, uv_plane) = NV12Organizer::get_nv12_planes(&nv12_data, width, height);
            println!("Y plane size: {} bytes", y_plane.len());
            println!("UV plane size: {} bytes", uv_plane.len());

            // 现在可以使用nv12_data进行视频编码等操作
        }
        Err(e) => {
            eprintln!("Error organizing NV12 data: {}", e);
        }
    }
}