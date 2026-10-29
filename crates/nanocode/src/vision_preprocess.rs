//! Local background-envelope correction. Preserve three luminance classes rather
//! than binarizing the two data inks into one. Only used after an ordinary miss.
pub(crate) fn flatten(gray: &[u8], width: usize, height: usize, invert: bool) -> Vec<u8> {
    let source: Vec<u8> = gray
        .iter()
        .map(|&v| if invert { 255 - v } else { v })
        .collect();
    let radius = (width.min(height) / 48).clamp(3, 24);
    let mut horizontal = vec![0u8; gray.len()];
    let mut envelope = vec![0u8; gray.len()];
    let mut queue = vec![0usize; width.max(height)];
    // Separable sliding maximum, O(pixels), independent of window area.
    for y in 0..height {
        maximum(
            &source[y * width..(y + 1) * width],
            &mut horizontal[y * width..(y + 1) * width],
            radius,
            &mut queue,
        );
    }
    let mut column = vec![0u8; height];
    let mut output = vec![0u8; height];
    for x in 0..width {
        for y in 0..height {
            column[y] = horizontal[y * width + x];
        }
        maximum(&column, &mut output, radius, &mut queue);
        for y in 0..height {
            envelope[y * width + x] = output[y];
        }
    }
    source
        .iter()
        .zip(envelope)
        .map(|(&v, bg)| {
            ((v as u32 * 240 + bg.max(32) as u32 / 2) / bg.max(32) as u32).min(255) as u8
        })
        .collect()
}
fn maximum(input: &[u8], output: &mut [u8], radius: usize, queue: &mut [usize]) {
    let (mut head, mut tail, mut right) = (0, 0, 0);
    for (x, out) in output.iter_mut().enumerate() {
        let end = (x + radius + 1).min(input.len());
        while right < end {
            while tail > head && input[queue[tail - 1]] <= input[right] {
                tail -= 1;
            }
            queue[tail] = right;
            tail += 1;
            right += 1;
        }
        while head < tail && queue[head] < x.saturating_sub(radius) {
            head += 1;
        }
        *out = input[queue[head]];
    }
}
