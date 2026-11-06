//! Shared atlas geometry only. Effects retain their own event clocks and timing.
use macroquad::prelude::*;

pub(super) fn frame(index: usize, columns: usize, size: Vec2) -> Rect {
    Rect::new(
        (index % columns) as f32 * size.x,
        (index / columns) as f32 * size.y,
        size.x,
        size.y,
    )
}

pub(super) fn clipped(
    index: usize,
    columns: usize,
    size: Vec2,
    origin: Vec2,
    clip: Rect,
    row_offset: usize,
) -> Option<(Vec2, Rect)> {
    let visible = Rect::new(origin.x, origin.y, size.x, size.y).intersect(clip)?;
    let cell = frame(index + row_offset * columns, columns, size);
    Some((
        vec2(visible.x, visible.y),
        Rect::new(
            cell.x + visible.x - origin.x,
            cell.y + visible.y - origin.y,
            visible.w,
            visible.h,
        ),
    ))
}

pub(super) fn draw(texture: &Texture2D, at: Vec2, source: Rect) {
    draw_texture_ex(
        texture,
        at.x,
        at.y,
        WHITE,
        DrawTextureParams {
            source: Some(source),
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clipping_preserves_frame_offsets_and_front_layers() {
        let (at, src) = clipped(
            9,
            8,
            vec2(160., 192.),
            vec2(-10., -20.),
            Rect::new(0., 0., 100., 100.),
            5,
        )
        .unwrap();
        assert_eq!(at, vec2(0., 0.));
        assert_eq!(src, Rect::new(170., 1172., 100., 100.));
        assert!(clipped(
            0,
            8,
            vec2(160., 192.),
            vec2(200., 200.),
            Rect::new(0., 0., 100., 100.),
            0
        )
        .is_none());
    }
}
