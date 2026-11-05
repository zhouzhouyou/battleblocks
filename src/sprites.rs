//! Character identity and appearance; generated artwork metadata lives separately.
use crate::content::{self, StatusSet};
mod animation;
pub use animation::*;
mod legacy;
pub use legacy::*;
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
pub const BOOK_BYTES: &[u8] = include_bytes!("../assets/roles/grudge-book.webp");
pub const BOOK_COVER: &[u8] = include_bytes!("../assets/roles/book-cover.webp");
pub const BOOK_PAGES: &[u8] = include_bytes!("../assets/roles/book-pages.webp");
pub const BOOK_BOTTOM: &[u8] = include_bytes!("../assets/roles/book-bottom.webp");
