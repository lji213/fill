use wasm_bindgen::prelude::*;
use std::collections::VecDeque;

#[wasm_bindgen]
pub fn flood_fill(
    width: u32,
    height: u32,
    pixels: &mut [u8],
    x: u32,
    y: u32,
    r: u8,
    g: u8,
    b: u8
) {
    let idx = |x: u32, y: u32| -> usize {
        ((y * width + x) * 4) as usize
    };

    if x >= width || y >= height {
        return;
    }

    let start = idx(x, y);
    let target = (
        pixels[start],
        pixels[start + 1],
        pixels[start + 2],
    );

    if target == (r, g, b) {
        return; // 이미 같은 색
    }

    let mut queue = VecDeque::new();
    queue.push_back((x, y));

    while let Some((cx, cy)) = queue.pop_front() {
        if cx >= width || cy >= height {
            continue;
        }

        let i = idx(cx, cy);
        let current = (
            pixels[i],
            pixels[i + 1],
            pixels[i + 2],
        );

        if current != target {
            continue;
        }

        pixels[i] = r;
        pixels[i + 1] = g;
        pixels[i + 2] = b;

        queue.push_back((cx + 1, cy));
        queue.push_back((cx.wrapping_sub(1), cy));
        queue.push_back((cx, cy + 1));
        queue.push_back((cx, cy.wrapping_sub(1)));
    }
}
