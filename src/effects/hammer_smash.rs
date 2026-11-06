//! One board impact at the center of Hammer's four cells, independent of caster side.
//! Offline Rust artwork: assets-source/effects/hammer-smash; MIT notices: licenses/pixelfx.
use macroquad::prelude::*;

const WIDTH: f32 = 192.;
const HEIGHT: f32 = 128.;
const COLUMNS: usize = 7;
const FRAMES: usize = 28;
const FPS: f32 = 30.;
const PIVOT: Vec2 = Vec2::new(96., 88.);
const BOARD: Rect = Rect::new(256., 110., 512., 512.);

struct Fragment {
    at: Vec2,
    source: Rect,
    alpha: f32,
}

fn fragment(
    x: usize,
    y: usize,
    column: usize,
    row: usize,
    center: Vec2,
    elapsed: f32,
) -> Option<Fragment> {
    let age = elapsed - crate::presentation::CLEAR_TIME as f32;
    if !(0.0..0.36).contains(&age) {
        return None;
    }
    let start = vec2(
        256. + x as f32 * 64. + column as f32 * 16.,
        110. + y as f32 * 64. + row as f32 * 16.,
    );
    let direction = (start + vec2(8., 8.) - center).normalize_or_zero();
    let seed = (x * 31 + y * 17 + column * 7 + row * 13) as f32;
    let vx = direction.x * (90. + seed.sin().abs() * 90.);
    let vy = -100. - (seed * 1.7).cos().abs() * 105. + direction.y * 75.;
    let origin = (start + vec2(vx * age, vy * age + 500. * age * age)).round();
    let visible = Rect::new(origin.x, origin.y, 16., 16.).intersect(BOARD)?;
    Some(Fragment {
        at: vec2(visible.x, visible.y),
        source: Rect::new(
            column as f32 * 16. + visible.x - origin.x,
            row as f32 * 16. + visible.y - origin.y,
            visible.w,
            visible.h,
        ),
        alpha: ((0.36 - age) / 0.16).clamp(0., 1.),
    })
}

pub(super) fn draw_fragments(
    tiles: &[Texture2D; 7],
    original: &[(usize, usize, u8)],
    cells: &[(usize, usize)],
    elapsed: f32,
) {
    if cells.is_empty() {
        return;
    }
    let center = cells.iter().fold(Vec2::ZERO, |sum, &(x, y)| {
        sum + vec2(288. + x as f32 * 64., 142. + y as f32 * 64.)
    }) / cells.len() as f32;
    for &(x, y, kind) in original {
        let Some(texture) = tiles.get(kind as usize) else {
            continue;
        };
        for row in 0..4 {
            for column in 0..4 {
                if let Some(piece) = fragment(x, y, column, row, center, elapsed) {
                    draw_texture_ex(
                        texture,
                        piece.at.x,
                        piece.at.y,
                        Color::new(1., 1., 1., piece.alpha),
                        DrawTextureParams {
                            source: Some(piece.source),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }
}

fn placement(cells: &[(usize, usize)], elapsed: f32) -> Option<(Vec2, Rect)> {
    if cells.is_empty() || !elapsed.is_finite() || !(0.0..0.9).contains(&elapsed) {
        return None;
    }
    let min_x = cells.iter().map(|p| p.0).min()?;
    let max_x = cells.iter().map(|p| p.0).max()?;
    let min_y = cells.iter().map(|p| p.1).min()?;
    let max_y = cells.iter().map(|p| p.1).max()?;
    let center = vec2(
        288. + (min_x + max_x) as f32 * 32.,
        142. + (min_y + max_y) as f32 * 32.,
    );
    let origin = (center - PIVOT).round();
    let index = ((elapsed * FPS).floor() as usize).min(FRAMES - 1);
    super::atlas::clipped(index, COLUMNS, vec2(WIDTH, HEIGHT), origin, BOARD, 0)
}

pub(super) fn draw(texture: &Texture2D, cells: &[(usize, usize)], elapsed: f32) {
    if let Some((at, source)) = placement(cells, elapsed) {
        super::atlas::draw(texture, at, source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;

    #[test]
    fn original_tile_pieces_exist_only_after_impact_and_stay_on_board() {
        assert!(crate::effects::holds_tiles(
            Some(crate::content::Skill::Hammer),
            0.399
        ));
        assert!(!crate::effects::holds_tiles(
            Some(crate::content::Skill::Hammer),
            0.4
        ));
        let center = vec2(320., 174.);
        assert!(fragment(0, 0, 0, 0, center, 0.399).is_none());
        let piece = fragment(0, 0, 0, 0, center, 0.4).unwrap();
        assert_eq!(piece.at, vec2(256., 110.));
        assert_eq!(piece.source, Rect::new(0., 0., 16., 16.));
        assert_eq!(piece.alpha, 1.);
        for frame in 12..24 {
            for row in 0..4 {
                for column in 0..4 {
                    if let Some(piece) = fragment(0, 0, column, row, center, frame as f32 / 30.) {
                        assert!(piece.at.x >= BOARD.x && piece.at.y >= BOARD.y);
                        assert!(piece.at.x + piece.source.w <= BOARD.right());
                        assert!(piece.at.y + piece.source.h <= BOARD.bottom());
                    }
                }
            }
        }
        assert!(fragment(0, 0, 0, 0, center, 0.77).is_none());
    }

    #[test]
    fn every_two_by_two_landing_stays_on_board_and_uses_one_frame() {
        for y in 0..7 {
            for x in 0..7 {
                let cells = [(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)];
                for frame in 0..27 {
                    let (at, source) = placement(&cells, frame as f32 / FPS + 0.001).unwrap();
                    assert!(at.x >= BOARD.x && at.y >= BOARD.y);
                    assert!(at.x + source.w <= BOARD.right() && at.y + source.h <= BOARD.bottom());
                    assert!(source.x >= (frame % COLUMNS) as f32 * WIDTH);
                    assert!(source.right() <= (frame % COLUMNS + 1) as f32 * WIDTH);
                    assert!(source.y >= (frame / COLUMNS) as f32 * HEIGHT);
                    assert!(source.bottom() <= (frame / COLUMNS + 1) as f32 * HEIGHT);
                }
            }
        }
        let cells = [(3, 3), (4, 3), (3, 4), (4, 4)];
        let (at, source) = placement(&cells, 0.4).unwrap();
        assert_eq!(at + PIVOT, vec2(512., 366.));
        assert_eq!(source, Rect::new(5. * WIDTH, HEIGHT, WIDTH, HEIGHT)); // Impact frame 12.
        assert!(placement(&cells, 0.9).is_none());
        assert!(placement(&[], 0.4).is_none());
    }

    #[test]
    fn baked_atlas_matches_runtime_layout_and_ends_empty() {
        let atlas =
            image::load_from_memory(include_bytes!("../../assets/effects/hammer-smash.webp"))
                .unwrap()
                .to_rgba8();
        assert_eq!(atlas.dimensions(), (1344, 512));
        let impact = atlas.view(5 * 192, 128, 192, 128).to_image();
        assert!(impact.pixels().any(|p| p.0[3] > 0));
        let last = atlas.view(6 * 192, 3 * 128, 192, 128).to_image();
        assert!(last.pixels().all(|p| p.0[3] == 0));
    }
}
