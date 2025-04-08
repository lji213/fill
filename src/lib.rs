use wasm_bindgen::prelude::*;
use std::collections::VecDeque;

// --- CIE L*a*b* Conversion Constants and Helpers ---
// D65 표준 광원 기준 XYZ 값
const REF_X: f64 = 95.047;
const REF_Y: f64 = 100.000;
const REF_Z: f64 = 108.883;

// sRGB (0-255) to Linear RGB (0.0-1.0) - 감마 보정 해제
fn srgb_to_linear(c: u8) -> f64 {
    let c_norm = c as f64 / 255.0;
    if c_norm <= 0.04045 {
        c_norm / 12.92
    } else {
        ((c_norm + 0.055) / 1.055).powf(2.4)
    }
}

// Linear RGB (0.0-1.0) to CIE XYZ
fn linear_rgb_to_xyz(rgb: [f64; 3]) -> [f64; 3] {
    let r = rgb[0];
    let g = rgb[1];
    let b = rgb[2];
    // sRGB to XYZ 변환 행렬 (D65) 적용 및 스케일링
    let x = (r * 0.4124564 + g * 0.3575761 + b * 0.1804375) * 100.0;
    let y = (r * 0.2126729 + g * 0.7151522 + b * 0.0721750) * 100.0;
    let z = (r * 0.0193339 + g * 0.1191920 + b * 0.9503041) * 100.0;
    [x, y, z]
}

// CIE XYZ to CIE L*a*b*
fn xyz_to_lab(xyz: [f64; 3]) -> [f64; 3] {
    // 기준 흰색으로 정규화
    let x = xyz[0] / REF_X;
    let y = xyz[1] / REF_Y;
    let z = xyz[2] / REF_Z;
    // 비선형 변환 함수 적용
    let fx = if x > 0.008856 { x.powf(1.0 / 3.0) } else { (903.3 * x + 16.0) / 116.0 };
    let fy = if y > 0.008856 { y.powf(1.0 / 3.0) } else { (903.3 * y + 16.0) / 116.0 };
    let fz = if z > 0.008856 { z.powf(1.0 / 3.0) } else { (903.3 * z + 16.0) / 116.0 };
    // L*, a*, b* 계산
    let l = (116.0 * fy) - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    [l.max(0.0), a, b] // L* 값은 0 이상 보장
}

// RGB 슬라이스(RGBA에서 추출)를 L*a*b* 배열로 변환하는 헬퍼
fn rgb_slice_to_lab(rgb: &[u8]) -> [f64; 3] {
    // 최소 3개 요소(R, G, B) 확인
    if rgb.len() < 3 {
        // 기본값 (검정색) 또는 적절한 오류 처리
        return xyz_to_lab(linear_rgb_to_xyz([0.0, 0.0, 0.0]));
    }
    let linear_r = srgb_to_linear(rgb[0]);
    let linear_g = srgb_to_linear(rgb[1]);
    let linear_b = srgb_to_linear(rgb[2]);
    let xyz = linear_rgb_to_xyz([linear_r, linear_g, linear_b]);
    xyz_to_lab(xyz)
}

// --- CIE76 Delta E 기반 색상 유사도 검사 함수 ---
// 입력: 두 RGBA 색상 슬라이스, 허용치(0-100)
// 동작: RGB 성분만 사용하여 CIE L*a*b* 공간에서의 거리(Delta E) 계산 후 허용치와 비교
fn is_similar_color(rgba1: &[u8], rgba2: &[u8], tolerance: u8) -> bool {
    // 두 슬라이스 모두 최소 RGB 데이터 필요
    if rgba1.len() < 3 || rgba2.len() < 3 {
        return false;
    }

    // 각 색상의 RGB 부분을 L*a*b*로 변환
    let lab1 = rgb_slice_to_lab(&rgba1[0..3]);
    let lab2 = rgb_slice_to_lab(&rgba2[0..3]);

    // L*a*b* 공간에서의 유클리디안 거리 제곱 계산 (CIE76 Delta E^2)
    let delta_l = lab1[0] - lab2[0];
    let delta_a = lab1[1] - lab2[1];
    let delta_b = lab1[2] - lab2[2];
    let delta_e_squared = delta_l * delta_l + delta_a * delta_a + delta_b * delta_b;

    // 허용치(0-100)를 Delta E 임계값(예: 0-50)으로 매핑하고 제곱
    let max_delta_e_threshold = tolerance as f64 * 1.01; // 예: tolerance 100 -> Delta E 50
    let max_delta_e_squared_threshold = max_delta_e_threshold * max_delta_e_threshold;

    // 거리 제곱값과 임계값 제곱값을 비교 (sqrt 계산 회피)
    delta_e_squared <= max_delta_e_squared_threshold
}


#[wasm_bindgen]
pub fn flood_fill(
    pixels: &[u8],
    width: u32,
    height: u32,
    start_x: u32,
    start_y: u32,
    fill_color: &[u8],
    tolerance: u8
) -> Vec<u8> {
    let mut data = pixels.to_vec();
    let width = width as usize;
    let height = height as usize;
    let start_x = start_x as usize;
    let start_y = start_y as usize;

    let get_idx = |x: usize, y: usize| (y * width + x) * 4;

    if start_x >= width || start_y >= height || fill_color.len() != 4 {
        eprintln!("Flood fill: 유효하지 않은 입력 매개변수입니다.");
        return data;
    }

    let start_idx = get_idx(start_x, start_y);
    let target_color: Vec<u8> = data[start_idx..start_idx + 4].to_vec();

    if target_color == fill_color {
        return data;
    }

    let mut visited = vec![false; width * height];
    let mut queue = VecDeque::new();

    // 시작점 유효성 검사
    if is_similar_color(&data[start_idx..start_idx+4], &target_color, tolerance) {
        queue.push_back((start_x, start_y));
        visited[start_y * width + start_x] = true;
    } else {
        return data;
    }

    // BFS
    while let Some((x, y)) = queue.pop_front() {
        let idx = get_idx(x, y);
        let current_color: Vec<u8> = data[idx..idx + 4].to_vec();

        if !is_similar_color(&current_color, &target_color, tolerance) {
            continue;
        }

        data[idx..idx + 4].copy_from_slice(fill_color);

        for (nx, ny) in [
            (x.wrapping_sub(1), y), (x + 1, y),
            (x, y.wrapping_sub(1)), (x, y + 1),
        ] {
            if nx < width && ny < height && !visited[ny * width + nx] {
                visited[ny * width + nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    data
}