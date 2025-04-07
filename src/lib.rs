use wasm_bindgen::prelude::*;
use std::collections::VecDeque;

#[wasm_bindgen]
pub fn flood_fill(
    pixels: &[u8],      // RGBA 픽셀 배열
    width: u32,
    height: u32,
    start_x: u32,
    start_y: u32,
    fill_color: &[u8],  // RGBA
    tolerance: u8       // 0 ~ 100 (유사도)
) -> Vec<u8> {
    let mut data = pixels.to_vec();
    let width = width as usize;
    let height = height as usize;
    let start_x = start_x as usize;
    let start_y = start_y as usize;

    let get_idx = |x: usize, y: usize| (y * width + x) * 4;

    if start_x >= width || start_y >= height || fill_color.len() != 4 {
        return data;
    }

    let start_idx = get_idx(start_x, start_y);
    let target_color = data[start_idx..start_idx + 4].to_vec();

    if target_color == fill_color {
        return data;
    }

    

    let mut visited = vec![false; width * height];
    let mut queue = VecDeque::new();

    queue.push_back((start_x, start_y));
    visited[start_y * width + start_x] = true;

    while let Some((x, y)) = queue.pop_front() {
        let idx = get_idx(x, y);
    
        let current_color = data[idx..idx + 4].to_vec(); // 복사본 생성 (참조 X)
    
        if !is_similar_color(&current_color, &target_color, tolerance) {
            continue;
        }
    
        data[idx..idx + 4].copy_from_slice(fill_color); 
    
        for (nx, ny) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if nx < width && ny < height && !visited[ny * width + nx] {
                visited[ny * width + nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    data
}


fn is_similar_color(a: &[u8], b: &[u8], tolerance: u8) -> bool {
    let dr = a[0] as i16 - b[0] as i16;
    let dg = a[1] as i16 - b[1] as i16;
    let db = a[2] as i16 - b[2] as i16;
    let da = a[3] as i16 - b[3] as i16;

    // 유클리디안 거리
    let distance_squared = dr * dr + dg * dg + db * db + da * da;

    // tolerance (0~100)을 0~441로 매핑 (RGB 거리 최대는 255 * sqrt(3) ≈ 441)
    let max_distance = (tolerance as f64 / 100.0) * 441.0;
    distance_squared as f64 <= max_distance * max_distance
}

