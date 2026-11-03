use crate::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Wrap using the same font metrics as labels and the available panel width.
/// The estimate keeps layout queries available without a GUI (headless/tests).
pub(crate) fn wrap_label_text(text: &str, size: f32, width: f32) -> Vec<String> {
    if UI_FONT.with(|slot| slot.borrow().is_some()) {
        wrap_measured_text(text, width, |line| label_width(line, size))
    } else {
        wrap_preview_text(text, width / size)
    }
}

pub(crate) fn wrap_preview_text(text: &str, limit: f32) -> Vec<String> {
    wrap_measured_text(text, limit, |text| {
        text.chars()
            .map(|c| if c.is_ascii() { 0.55 } else { 1. })
            .sum()
    })
}

fn wrap_measured_text(text: &str, limit: f32, measure: impl Fn(&str) -> f32) -> Vec<String> {
    let mut lines = Vec::new();
    for source in text.lines() {
        let mut line = String::new();
        for c in source.chars() {
            let mut candidate = line.clone();
            candidate.push(c);
            if c.is_whitespace() && measure(&candidate) > limit && !line.is_empty() {
                lines.push(line.trim_end().to_string());
                line.clear();
                continue;
            }
            // Prefer word boundaries; long words and CJK still wrap by character.
            while measure(&candidate) > limit && !line.is_empty() {
                if let Some(space) = line.rfind(' ') {
                    let rest = line.split_off(space + 1);
                    let head = line.trim_end().to_string();
                    if !head.is_empty() {
                        lines.push(head);
                    }
                    line = rest.trim_start().to_string();
                } else {
                    lines.push(std::mem::take(&mut line));
                }
                candidate = line.clone();
                candidate.push(c);
            }
            // A space at the wrap boundary belongs between lines, not at the start.
            if !line.is_empty() || !c.is_whitespace() {
                line.push(c);
            }
        }
        if !line.trim_end().is_empty() {
            lines.push(line.trim_end().to_string());
        }
    }
    lines
}
pub(crate) fn move_hints_rect() -> Rect {
    Rect::new(796., 648., 208., 44.)
}
pub(crate) fn reroll_rect() -> Rect {
    Rect::new(256., 650., 512., 38.)
}
pub(crate) fn cancel_skill_rect() -> Rect {
    move_hints_rect()
}
// Battle sidebars: the player on the left, the enemy on the right. Every element
// below stays inside its sidebar frame.
pub(crate) fn side_panel_rect(enemy: bool) -> Rect {
    Rect::new(if enemy { 788. } else { 12. }, 90., 224., 610.)
}
pub(crate) fn sprite_rect(enemy: bool) -> Rect {
    Rect::new(if enemy { 796. } else { 20. }, 188., 208., 184.)
}
pub(crate) fn passive_rect(enemy: bool) -> Rect {
    Rect::new(if enemy { 796. } else { 20. }, 376., 208., 50.)
}
// Cards keep three rows (number + name, cost, "does not end turn") and leave the
// description to the hover panel; the pocket strip sits under the last card.
pub(crate) fn player_skill_rect(i: usize) -> Rect {
    Rect::new(20., 460. + i as f32 * 70., 208., 66.)
}
pub(crate) fn enemy_skill_rect(i: usize) -> Rect {
    Rect::new(796., 470. + i as f32 * 44., 208., 38.)
}
pub(crate) fn pocket_rect(enemy: bool, slot: usize) -> Rect {
    Rect::new(
        if enemy { 796. } else { 20. } + slot as f32 * 105.,
        if enemy { 610. } else { 670. },
        100.,
        25.,
    )
}
pub(crate) fn draw_cooldown_overlay(rect: Rect, remaining: u8) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0., 0., 0., 0.72));
    let center = vec2(rect.x + rect.w / 2., rect.y + rect.h / 2.);
    let radius = if rect.h < 50. { 15. } else { 25. };
    draw_circle(
        center.x,
        center.y,
        radius,
        Color::from_rgba(25, 32, 42, 245),
    );
    draw_circle_lines(center.x, center.y, radius, 2., GOLD);
    let size = if rect.h < 50. { 18. } else { 27. };
    label_centered(
        &remaining.to_string(),
        center.x,
        center.y + size * 0.36,
        size,
        WHITE,
    );
}
// Turn indicator rows above the board frame, between the two sidebars.
pub(crate) fn previous_turn_rect() -> Rect {
    Rect::new(236., 6., 552., 32.)
}
pub(crate) fn current_turn_rect() -> Rect {
    Rect::new(236., 42., 552., 42.)
}
pub(crate) type Move = ((usize, usize), (usize, usize));
/// Row of the move-reward panel under the pointer; the panel lists 7 hints per page.
pub(crate) fn hint_row_at(pointer: Vec2, page: usize, count: usize) -> Option<usize> {
    // Half-open bounds: `Rect::contains` includes the bottom edge, which would reach a
    // row of the next page.
    (pointer.x >= 798. && pointer.x < 1002. && pointer.y >= 278. && pointer.y < 586.)
        .then(|| page * 7 + ((pointer.y - 278.) / 44.) as usize)
        .filter(|&i| i < count)
}
/// Numbered hint badge on the board under the pointer, nearest first.
pub(crate) fn hint_badge_at(pointer: Vec2, hints: &[Move]) -> Option<usize> {
    if !(pointer.x >= 256. && pointer.x < 768. && pointer.y >= 110. && pointer.y < 622.) {
        return None;
    }
    hints
        .iter()
        .enumerate()
        .map(|(i, &(a, c))| (i, presentation::move_hint_anchor(a, c).distance(pointer)))
        .filter(|(_, d)| *d < 22.)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}
/// Reward part of the action feedback line, in the UI language.
pub(crate) fn reward_summary(zh: bool, heal: i32, damage: i32, mana: i32) -> String {
    format_text(
        zh,
        "combat.reward_summary",
        &[&heal.to_string(), &damage.to_string(), &mana.to_string()],
    )
}
/// Skill cost in the UI language, including the HP part of a flesh-casting payment.
pub(crate) fn draw_diamond(x: f32, y: f32, radius: f32, color: Color, filled: bool) {
    let points = [
        vec2(x, y - radius),
        vec2(x + radius, y),
        vec2(x, y + radius),
        vec2(x - radius, y),
    ];
    if filled {
        draw_triangle(points[0], points[1], points[2], color);
        draw_triangle(points[0], points[2], points[3], color);
    } else {
        for i in 0..4 {
            draw_line(
                points[i].x,
                points[i].y,
                points[(i + 1) % 4].x,
                points[(i + 1) % 4].y,
                1.5,
                color,
            );
        }
    }
}
pub(crate) fn draw_breaking_lock(x: f32, y: f32, t: f32, color: Color) {
    let split = ((t - 0.15) / 0.6).clamp(0., 1.);
    if split == 0. {
        draw_lock(x, y, color);
        return;
    }
    let c = Color::new(color.r, color.g, color.b, 1. - split);
    for (direction, offset) in [(-1., 0.), (1., 6.)] {
        let dx = direction * split * 14.;
        let dy = -split * 7. + split * split * 17.;
        draw_rectangle(x + offset + dx, y + 6. + dy, 5., 8., c);
        draw_line(
            x + offset + 2. + dx,
            y + 7. + dy,
            x + offset + 2. + dx,
            y + dy,
            2.,
            c,
        );
    }
}
/// World units per device pixel for the camera the UI draws through.
pub(crate) fn device_scale() -> f32 {
    (screen_width() / 1024.)
        .min(screen_height() / 720.)
        .max(0.001)
}
fn snap_to_pixel(value: f32, scale: f32) -> f32 {
    (value * scale).round() / scale
}
/// Rectangle outline that is guaranteed to stay visible at every window size.
///
/// `draw_rectangle_lines` draws a half-thickness outline inside the rectangle, so a
/// "1px" border can fall between two pixel centers and vanish completely when the
/// window scale is fractional. That is why borders kept disappearing on one side
/// (most often the right one). Drawing the four edges as filled strips, snapped to
/// whole device pixels, keeps every edge crisp and present at any scale.
pub(crate) fn outline(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    let scale = device_scale();
    let left = snap_to_pixel(x, scale);
    let top = snap_to_pixel(y, scale);
    let right = snap_to_pixel(x + w, scale);
    let bottom = snap_to_pixel(y + h, scale);
    let (w, h) = (right - left, bottom - top);
    if w <= 0. || h <= 0. {
        return;
    }
    // At least one device pixel wide, never thicker than the smaller half side.
    let t = ((thickness * scale).round().max(1.) / scale).min(w.min(h) / 2.);
    draw_rectangle(left, top, w, t, color);
    draw_rectangle(left, bottom - t, w, t, color);
    draw_rectangle(left, top + t, t, h - 2. * t, color);
    draw_rectangle(right - t, top + t, t, h - 2. * t, color);
}
pub(crate) fn draw_lock(x: f32, y: f32, color: Color) {
    outline(x + 2., y, 7., 8., 2., color);
    draw_rectangle(x, y + 6., 11., 8., color);
    draw_rectangle(x + 5., y + 8., 2., 4., Color::from_rgba(25, 33, 46, 255));
}
pub(crate) fn cost_text(zh: bool, mana: i32, blood: i32) -> String {
    let mana = mana.to_string();
    if blood > 0 {
        let blood = blood.to_string();
        format_text(zh, "skill.cost_with_blood", &[&mana, &blood])
    } else {
        format_text(zh, "skill.cost_mana_only", &[&mana])
    }
}
#[cfg(test)]
pub(crate) fn skill_requires_board_click(skill: content::Skill) -> bool {
    content::targeted(skill)
}
pub(crate) fn toggle_skill_target(
    current: Option<content::Skill>,
    skill: content::Skill,
) -> Option<content::Skill> {
    if current == Some(skill) {
        None
    } else {
        Some(skill)
    }
}
pub(crate) fn diagnostic(flag: &str) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static FLAGS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        FLAGS
            .get_or_init(|| std::env::var("BATTLEBLOCKS_MEMORY_FLAGS").unwrap_or_default())
            .split(',')
            .any(|s| s == flag)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = flag;
        false
    }
}
pub(crate) fn diagnostic_texture(bytes: &[u8], flag: &str) -> Texture2D {
    if diagnostic(flag) {
        Texture2D::empty()
    } else {
        texture(bytes)
    }
}
pub(crate) fn procedural_board() {
    let dark = Color::from_rgba(43, 37, 33, 255);
    let mid = Color::from_rgba(172, 145, 126, 255);
    let light = Color::from_rgba(214, 193, 167, 255);
    draw_rectangle(236., 90., 552., 552., dark);
    draw_rectangle(241., 95., 542., 542., mid);
    outline(241., 95., 542., 542., 3., light);
    for i in 0..18 {
        let p = i as f32 * 30.;
        draw_line(241. + p, 95., 241. + p, 108., 3., dark);
        draw_line(241. + p, 624., 241. + p, 637., 3., dark);
        draw_line(241., 95. + p, 254., 95. + p, 3., dark);
        draw_line(770., 95. + p, 783., 95. + p, 3., dark);
    }
}
pub(crate) fn procedural_cell(x: f32, y: f32) {
    draw_rectangle(x, y, 64., 64., Color::from_rgba(172, 127, 115, 255));
    outline(x, y, 64., 64., 3., Color::from_rgba(43, 37, 33, 255));
    outline(
        x + 4.,
        y + 4.,
        56.,
        56.,
        2.,
        Color::from_rgba(214, 193, 167, 255),
    );
}
pub(crate) fn texture(bytes: &[u8]) -> Texture2D {
    let t = Texture2D::from_file_with_format(bytes, None);
    t.set_filter(FilterMode::Nearest);
    t
}
struct LabelJob {
    text: String,
    x: f32,
    y: f32,
    font_size: u16,
    font_scale: f32,
    font_scale_aspect: f32,
    color: Color,
    // Share of the measured width drawn left of `x`: 0 left-aligns, 0.5 centers.
    align: f32,
}
struct UiFont {
    font: Font,
    cached: std::collections::HashSet<(char, u16)>,
    scratch: Vec<char>,
    jobs: Vec<LabelJob>,
    pending: usize,
}
thread_local! { static UI_FONT: RefCell<Option<UiFont>> = const { RefCell::new(None) }; }
type LocalizationTable = HashMap<&'static str, [String; 2]>;
static LOCALIZATION: OnceLock<LocalizationTable> = OnceLock::new();
fn localization() -> &'static LocalizationTable {
    LOCALIZATION.get_or_init(|| {
        let mut table = HashMap::new();
        for (line_number, line) in include_str!("../assets/localization.tsv")
            .lines()
            .enumerate()
        {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut fields = line.splitn(3, '\t');
            let key = fields.next().expect("localization key");
            let en = fields.next().expect("English localization");
            let zh = fields.next().expect("Chinese localization");
            let unescape = |value: &str| value.replace("\\n", "\n").replace("\\t", "\t");
            assert!(
                table.insert(key, [unescape(en), unescape(zh)]).is_none(),
                "duplicate localization key on line {}: {key}",
                line_number + 1
            );
        }
        table
    })
}
/// Look up UI copy by stable key. Missing entries fail immediately during development.
pub(crate) fn tr_key(zh: bool, key: &'static str) -> &'static str {
    let row = localization()
        .get(key)
        .unwrap_or_else(|| panic!("missing localization key: {key}"));
    if zh {
        &row[1]
    } else {
        &row[0]
    }
}
/// Format a localized resource template using positional placeholders such as `{0}` and `{1}`.
pub(crate) fn format_text(zh: bool, key: &'static str, args: &[&str]) -> String {
    let mut text = tr_key(zh, key).to_owned();
    for (index, value) in args.iter().enumerate() {
        text = text.replace(&format!("{{{index}}}"), value);
    }
    text
}
pub(crate) fn localized_note(zh: bool, note: &str) -> &str {
    let key = match note {
        "note.skill_cast" => "note.skill_cast",
        "note.not_enough_mana" => "note.not_enough_mana",
        "note.no_moves" => "note.no_moves",
        "note.match" => "note.match",
        "note.no_match" => "note.no_match",
        "note.select_adjacent_tiles" => "note.select_adjacent_tiles",
        _ => return note,
    };
    tr_key(zh, key)
}

pub(crate) fn font_raster_bucket(requested: u16) -> u16 {
    [16u16, 24, 32, 48, 64]
        .into_iter()
        .find(|&size| size >= requested)
        .unwrap_or(64)
}
pub(crate) fn label(text: &str, x: f32, y: f32, size: f32, color: Color) {
    queue_label(text, x, y, size, color, 0.);
}
pub(crate) fn queue_label(text: &str, x: f32, y: f32, size: f32, color: Color, align: f32) {
    if diagnostic("no_text") {
        return;
    }
    let (requested, font_scale, font_scale_aspect) = camera_font_scale(size);
    let font_size = font_raster_bucket(requested);
    let font_scale = font_scale * requested.max(1) as f32 / font_size as f32;
    UI_FONT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(ui) = slot.as_mut() {
            let n = ui.pending;
            ui.pending += 1;
            if n == ui.jobs.len() {
                ui.jobs.push(LabelJob {
                    text: String::new(),
                    x,
                    y,
                    font_size,
                    font_scale,
                    font_scale_aspect,
                    color,
                    align,
                });
            }
            let job = &mut ui.jobs[n];
            job.text.clear();
            job.text.push_str(text);
            job.x = x;
            job.y = y;
            job.font_size = font_size;
            job.font_scale = font_scale;
            job.font_scale_aspect = font_scale_aspect;
            job.color = color;
            job.align = align;
        }
    });
}
/// Label width in world units. Measure before the frame's first `flush_labels`, so caching
/// a new glyph cannot grow the atlas under text that was already drawn this frame.
pub(crate) fn label_width(text: &str, size: f32) -> f32 {
    if diagnostic("no_text") {
        return 0.;
    }
    let (requested, font_scale, _) = camera_font_scale(size);
    let font_size = font_raster_bucket(requested);
    let font_scale = font_scale * requested.max(1) as f32 / font_size as f32;
    UI_FONT.with(|slot| {
        slot.borrow().as_ref().map_or(0., |ui| {
            measure_text(text, Some(&ui.font), font_size, font_scale).width
        })
    })
}
/// Font size that fits `text` into `width`, never larger than `size`.
pub(crate) fn fit_size(text: &str, size: f32, width: f32) -> f32 {
    (size * width / label_width(text, size).max(1.)).min(size)
}
/// Differently colored pieces of one line from `x`, shrunk together to fit into `room`.
/// Measures like `label_width`, so draw before the frame's first `flush_labels`.
pub(crate) fn label_segments(segments: &[(String, Color)], x: f32, y: f32, size: f32, room: f32) {
    let width: f32 = segments
        .iter()
        .map(|(text, _)| label_width(text, size))
        .sum();
    let size = (size * room / width.max(1.)).min(size);
    let mut x = x;
    for (text, color) in segments {
        label(text, x, y, size, *color);
        x += label_width(text, size);
    }
}
/// Centered on `center_x`. The width is measured when the label is flushed, after the
/// frame's glyphs are cached, so this is safe after an earlier `flush_labels` too.
pub(crate) fn label_centered(text: &str, center_x: f32, y: f32, size: f32, color: Color) {
    queue_label(text, center_x, y, size, color, 0.5);
}
pub(crate) fn flush_labels() {
    UI_FONT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(ui) = slot.as_mut() {
            let UiFont {
                font,
                cached,
                scratch,
                jobs,
                pending,
            } = ui;
            // Populate the whole frame before rendering any font texture.
            // Atlas growth must not invalidate text batches already queued this frame.
            for job in jobs.iter().take(*pending) {
                let raster_size =
                    (job.font_size as f32 * miniquad::window::dpi_scale()).ceil() as u16;
                scratch.clear();
                for c in job.text.chars() {
                    if cached.insert((c, raster_size)) {
                        scratch.push(c);
                    }
                }
                if !scratch.is_empty() {
                    font.populate_font_cache(scratch, raster_size);
                }
            }
            for job in jobs.iter().take(*pending) {
                let x = if job.align > 0. {
                    let width =
                        measure_text(&job.text, Some(font), job.font_size, job.font_scale).width;
                    job.x - width * job.align
                } else {
                    job.x
                };
                draw_text_ex(
                    &job.text,
                    x,
                    job.y,
                    TextParams {
                        font: Some(font),
                        font_size: job.font_size,
                        font_scale: job.font_scale,
                        font_scale_aspect: job.font_scale_aspect,
                        color: job.color,
                        ..Default::default()
                    },
                );
            }
            *pending = 0;
        }
    });
}

/// New atlas frames share a 96px canvas but their body ends at y=84.
/// Fit visible equipment bounds rather than shrinking the transparent margins.
pub(crate) fn role_scale(area: Rect) -> f32 {
    (area.w / 84.).min(area.h / 83.).min(3.)
}
pub(crate) fn actor(
    t: &Texture2D,
    w: f32,
    h: f32,
    area: Rect,
    time: f64,
    durations: &[f64],
    indices: &[usize],
) {
    if diagnostic("no_actors") {
        return;
    }
    let total: f64 = durations.iter().sum();
    let mut elapsed = time % total;
    let mut frame = 0;
    for (i, d) in durations.iter().enumerate() {
        frame = i;
        if elapsed < *d {
            break;
        }
        elapsed -= d;
    }
    // Fit the frame into its slot and stand it on the slot's bottom edge.
    let new_role = w == 384. && h == 384.;
    let scale = if new_role {
        role_scale(area) * 96. / w
    } else {
        (area.w / w)
            .min(area.h / h)
            .min(if h <= 96. { 3. } else { 0.55 })
    };
    let size = vec2(w * scale, h * scale);
    let (size, rotation) = presentation::idle_pose(size, time, indices.len() == 1);
    let foot = vec2(area.x + area.w / 2., area.y + area.h);
    draw_texture_ex(
        t,
        area.x + (area.w - size.x) / 2.,
        if new_role {
            area.y + area.h - 84. * (w / 96.) * scale
        } else {
            area.y + area.h - size.y
        },
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            source: Some(Rect::new(
                (indices[frame] % (t.width() / w) as usize) as f32 * w,
                (indices[frame] / (t.width() / w) as usize) as f32 * h,
                w,
                h,
            )),
            rotation,
            pivot: Some(foot),
            ..Default::default()
        },
    );
}

pub(crate) fn init_font() {
    if !diagnostic("no_text") {
        let mut font =
            load_ttf_font_from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/ui-subset.ttf")))
                .expect("UI font");
        font.set_filter(FilterMode::Linear);
        UI_FONT.with(|slot| {
            *slot.borrow_mut() = Some(UiFont {
                font,
                cached: std::collections::HashSet::new(),
                scratch: Vec::new(),
                jobs: Vec::new(),
                pending: 0,
            })
        });
    }
}

#[cfg(test)]
mod wrapping_tests {
    use super::*;

    #[test]
    fn measured_wrapping_uses_width_instead_of_character_count() {
        let measure = |s: &str| s.chars().map(|c| if c == 'W' { 10. } else { 2. }).sum();
        assert_eq!(wrap_measured_text("WWii", 24., measure), ["WWii"]);
        assert_eq!(wrap_measured_text("WWii", 12., measure), ["W", "Wi", "i"]);
        assert_eq!(
            wrap_measured_text("one two three", 14., measure),
            ["one two", "three"]
        );
        assert_eq!(
            wrap_measured_text("one\ntwo", 100., measure),
            ["one", "two"]
        );
    }
}
