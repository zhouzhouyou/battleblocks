//! Character identity and appearance; generated artwork metadata lives separately.
use crate::content::{self, StatusSet};
mod animation;
pub use animation::*;
mod legacy;
pub use legacy::*;
mod portal_cube;
pub use portal_cube::{PORTAL_AWAKE, PORTAL_NORMAL};
mod registry;
pub use registry::{AWAKENED, SPRITES};
pub fn shadow_form(hero: usize, awake_visible: bool, statuses: StatusSet) -> bool {
    let mut dark = awake_visible;
    content::dispatch_rules(
        &content::EventContext::new(
            hero,
            0,
            0,
            content::Rules {
                awake: awake_visible,
                statuses,
                ..Default::default()
            },
        ),
        &mut content::RuleEvent::Appearance { dark: &mut dark },
        &mut Vec::new(),
    );
    dark
}
#[cfg(test)]
mod nun_visual_tests {
    use super::*;
    use content::{Skill, StatusDuration, StatusId};
    #[test]
    fn nun_appearance_follows_permanent_and_temporary_effects() {
        let mut s = StatusSet::default();
        assert!(!shadow_form(content::NUN, false, s));
        s.insert(
            StatusId::Frenzy,
            Skill::Frenzy,
            StatusDuration::OwnerTurnEnd(1),
        );
        assert!(shadow_form(content::NUN, false, s));
        s = StatusSet::default();
        assert!(shadow_form(content::NUN, true, s));
        s.insert(
            StatusId::Rational,
            Skill::Rational,
            StatusDuration::OwnerTurnEnd(1),
        );
        assert!(!shadow_form(content::NUN, true, s));
    }
}

#[cfg(test)]
pub const BOOK_BYTES: &[u8] = include_bytes!("../assets/roles/dwarf/runtime/grudge-book.webp");
pub const BOOK_COVER: &[u8] = include_bytes!("../assets/roles/dwarf/runtime/book-cover.webp");
pub const BOOK_PAGES: &[u8] = include_bytes!("../assets/roles/dwarf/runtime/book-pages.webp");
pub const BOOK_BOTTOM: &[u8] = include_bytes!("../assets/roles/dwarf/runtime/book-bottom.webp");

/// Placeholder art aliases an existing thumbnail without changing the hero identity.
pub fn thumbnail_index(hero: usize) -> usize {
    if hero == content::SLINGSHOT_USER {
        3
    } else {
        hero
    }
}

/// The same skill-specific cast selection serves the menu and both battle sides.
/// Registered overrides live with the artwork, not in combat rules.
pub fn cast_clip(hero: usize, awake: bool, skill: Option<content::Skill>) -> Option<AnimationClip> {
    let asset = if awake {
        &AWAKENED[hero]
    } else {
        &SPRITES[hero]
    };
    skill
        .and_then(|skill| asset.actor_cast(skill).map(|spec| spec.clip))
        .or_else(|| {
            skill.and_then(|skill| {
                portal_cube::SKILL_CASTS
                    .iter()
                    .find(|&&(id, phase, s, _)| id == hero && phase == awake && s == skill)
                    .map(|entry| entry.3)
            })
        })
        .or_else(|| asset.action(skill == Some(content::Skill::Jump)))
}
