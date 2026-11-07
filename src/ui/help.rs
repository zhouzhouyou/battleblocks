//! Read-only teaching diagrams; no simulation, random sampling, or new artwork.
use super::*;

fn text(zh: bool, key: &'static str, x: f32, y: f32, width: f32, size: f32, color: Color) {
    let value = tr_key(zh, key);
    label(value, x, y, fit_size(value, size, width), color);
}
fn arrow(from: Vec2, to: Vec2, color: Color) {
    draw_line(from.x, from.y, to.x, to.y, 2., color);
    let direction = (to - from).normalize();
    let normal = vec2(-direction.y, direction.x);
    for sign in [-1., 1.] {
        let wing = to - direction * 7. + normal * sign * 4.;
        draw_line(to.x, to.y, wing.x, wing.y, 2., color);
    }
}
fn tile(tiles: &[Texture2D], kind: Option<usize>, x: f32, y: f32, size: f32) {
    draw_rectangle(x, y, size, size, Color::from_rgba(16, 22, 33, 255));
    outline(x, y, size, size, 1., GRAY);
    if let Some(kind) = kind {
        draw_texture_ex(
            &tiles[kind],
            x + 2.,
            y + 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size - 4., size - 4.)),
                ..Default::default()
            },
        );
    }
}
fn rules(zh: bool, entries: &[(&'static str, &'static str)], top: f32) {
    for (i, (first, second)) in entries.iter().enumerate() {
        let y = top + i as f32 * 40.;
        text(zh, first, PANEL_LEFT, y, PANEL_INNER, 17., WHITE);
        text(zh, second, PANEL_LEFT, y + 19., PANEL_INNER, 16., LIGHTGRAY);
    }
}
pub(super) fn draw(tab: usize, zh: bool, tiles: &[Texture2D]) {
    if tab == 3 {
        gravity(zh, tiles);
    } else {
        portals(zh, tiles);
    }
    text(
        zh,
        "ui.screen.draw_panel.esc_close",
        PANEL_LEFT,
        PANEL.y + PANEL.h - 12.,
        PANEL_INNER,
        16.,
        LIGHTGRAY,
    );
}
fn gravity(zh: bool, tiles: &[Texture2D]) {
    text(
        zh,
        "help.gravity.title",
        PANEL_LEFT,
        221.,
        PANEL_INNER,
        22.,
        GOLD,
    );
    text(
        zh,
        "help.gravity.quadrants",
        PANEL_LEFT,
        250.,
        238.,
        17.,
        SKYBLUE,
    );
    // The exact same quadrant mapping as Gravity::from_quadrant.
    for (i, (key, direction)) in [
        ("help.gravity.up", vec2(0., -1.)),
        ("help.gravity.right", vec2(1., 0.)),
        ("help.gravity.left", vec2(-1., 0.)),
        ("help.gravity.down", vec2(0., 1.)),
    ]
    .into_iter()
    .enumerate()
    {
        let r = Rect::new(
            PANEL_LEFT + (i % 2) as f32 * 112.,
            263. + (i / 2) as f32 * 70.,
            108.,
            66.,
        );
        frame(r, false);
        let center = vec2(r.x + r.w / 2., r.y + 22.);
        arrow(center - direction * 12., center + direction * 12., GOLD);
        let value = tr_key(zh, key);
        label_centered(
            value,
            center.x,
            r.y + 55.,
            fit_size(value, 16., r.w - 12.),
            WHITE,
        );
    }
    // A horizontal example makes clear that settling is no longer always downward.
    let x = PANEL_LEFT + 276.;
    text(zh, "help.gravity.before", x, 274., 276., 17., WHITE);
    for (i, kind) in [Some(2), None, Some(3), Some(1)].into_iter().enumerate() {
        tile(tiles, kind, x + i as f32 * 48., 288., 42.);
    }
    arrow(vec2(x + 174., 347.), vec2(x + 20., 347.), GOLD);
    for (i, kind) in [2, 3, 1, 0].into_iter().enumerate() {
        tile(tiles, Some(kind), x + i as f32 * 48., 364., 42.);
    }
    outline(x + 144., 364., 42., 42., 2., SKYBLUE);
    text(zh, "help.gravity.after", x, 429., 276., 17., SKYBLUE);
    rules(
        zh,
        &[
            ("help.gravity.rule1", "help.gravity.rule1b"),
            ("help.gravity.rule2", "help.gravity.rule2b"),
            ("help.gravity.rule3", "help.gravity.rule3b"),
        ],
        464.,
    );
}
fn portals(zh: bool, tiles: &[Texture2D]) {
    text(
        zh,
        "help.portals.title",
        PANEL_LEFT,
        221.,
        PANEL_INNER,
        22.,
        GOLD,
    );
    text(
        zh,
        "help.portals.diagram",
        PANEL_LEFT,
        246.,
        PANEL_INNER,
        17.,
        LIGHTGRAY,
    );
    // Two cut columns: blue upstream connects to orange downstream and vice versa.
    // The two left upstream tiles and right downstream tile form a cross-gate match.
    let left = PANEL_LEFT + 98.;
    let right = PANEL_LEFT + 394.;
    for (x, kinds) in [(left, [2, 2, 1]), (right, [3, 0, 2])] {
        for (row, kind) in kinds.into_iter().enumerate() {
            let y = [254., 293., 375.][row];
            tile(tiles, Some(kind), x, y, 34.);
            if kind == 2 {
                outline(x, y, 34., 34., 2., GOLD);
            }
        }
    }
    for (x, color) in [(left, SKYBLUE), (right, ORANGE)] {
        draw_rectangle(x - 5., 337., 44., 6., color);
        draw_rectangle(x - 3., 339., 40., 2., WHITE);
        arrow(vec2(x - 22., 270.), vec2(x - 22., 322.), GOLD);
    }
    // Both links continue downstream; a dark under-stroke separates the crossing.
    draw_line(left + 17., 343., right + 17., 364., 2., SKYBLUE);
    arrow(vec2(right + 17., 364.), vec2(right + 17., 374.), SKYBLUE);
    draw_line(
        right + 17.,
        343.,
        left + 17.,
        364.,
        6.,
        Color::from_rgba(24, 30, 43, 255),
    );
    draw_line(right + 17., 343., left + 17., 364., 2., ORANGE);
    arrow(vec2(left + 17., 364.), vec2(left + 17., 374.), ORANGE);
    text(
        zh,
        "help.portals.match",
        PANEL_LEFT,
        435.,
        PANEL_INNER,
        17.,
        GOLD,
    );
    rules(
        zh,
        &[
            ("help.portals.rule1", "help.portals.rule1b"),
            ("help.portals.rule2", "help.portals.rule2b"),
            ("help.portals.rule3", "help.portals.rule3b"),
            ("help.portals.rule4", "help.portals.rule4b"),
        ],
        456.,
    );
}
