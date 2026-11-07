use super::*;

/// Character-file text shown on the final selection page.
pub struct RoleLore {
    title_en: String,
    title_zh: String,
    intro_en: String,
    intro_zh: String,
    traits: Vec<(String, String)>,
    lines: Vec<(String, String)>,
    voices: Vec<(String, usize)>,
}
impl RoleLore {
    pub fn title(&self, zh: bool) -> &str {
        if zh {
            &self.title_zh
        } else {
            &self.title_en
        }
    }
    pub fn intro(&self, zh: bool) -> &str {
        if zh {
            &self.intro_zh
        } else {
            &self.intro_en
        }
    }
    pub fn trait_text(&self, index: usize, zh: bool) -> Option<&str> {
        self.traits
            .get(index)
            .map(|(en, zh_text)| if zh { zh_text } else { en }.as_str())
    }
    /// Voice cues reuse the character's configured dialogue, without its display label.
    pub fn voice(&self, cue: &str, zh: bool) -> Option<&str> {
        let (_, index) = self.voices.iter().find(|(key, _)| key == cue)?;
        let (en, cn) = &self.lines[*index];
        let text = if zh { cn } else { en };
        Some(
            text.split_once(" · ")
                .map_or(text.as_str(), |(_, line)| line),
        )
    }
    pub fn lines(&self, zh: bool) -> impl Iterator<Item = &str> {
        self.lines
            .iter()
            .map(move |(en, zh_text)| if zh { zh_text } else { en }.as_str())
    }
}
pub fn role_lore(hero: usize) -> Option<&'static RoleLore> {
    static DATA: std::sync::OnceLock<Vec<RoleLore>> = std::sync::OnceLock::new();
    DATA.get_or_init(|| {
        let mut data: Vec<RoleLore> = (0..HEROES.len())
            .map(|_| RoleLore {
                title_en: String::new(),
                title_zh: String::new(),
                intro_en: String::new(),
                intro_zh: String::new(),
                traits: Vec::new(),
                lines: Vec::new(),
                voices: Vec::new(),
            })
            .collect();
        for line in include_str!("../../config/role-lore.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let fields: Vec<_> = line.split('|').collect();
            assert_eq!(fields.len(), 4, "invalid role lore row: {line}");
            let hero: usize = fields[0].parse().expect("role lore hero");
            let entry = data.get_mut(hero).expect("role lore hero in range");
            let en = fields[2].replace("\\n", "\n");
            let zh = fields[3].replace("\\n", "\n");
            match fields[1] {
                "title" => {
                    entry.title_en = en;
                    entry.title_zh = zh;
                }
                "intro" => {
                    entry.intro_en = en;
                    entry.intro_zh = zh;
                }
                "trait" => entry.traits.push((en, zh)),
                "line" => entry.lines.push((en, zh)),
                kind if kind.starts_with("voice.") => {
                    let cue = &kind[6..];
                    assert!(
                        !entry.voices.iter().any(|(key, _)| key == cue),
                        "duplicate voice cue"
                    );
                    entry.voices.push((cue.to_owned(), entry.lines.len()));
                    entry.lines.push((en, zh));
                }
                kind => panic!("unknown role lore kind: {kind}"),
            }
        }
        for (hero, entry) in data.iter().enumerate() {
            assert!(
                !entry.title_en.is_empty() && !entry.intro_en.is_empty(),
                "missing role lore for hero {hero}"
            );
        }
        data
    });
    DATA.get().and_then(|data| data.get(hero))
}

#[cfg(test)]
mod voice_tests {
    use super::*;
    #[test]
    fn configured_voice_cues_are_bilingual_and_reuse_display_dialogue() {
        let lore = role_lore(12).unwrap();
        for cue in ["victory", "defeat"] {
            for zh in [false, true] {
                let voice = lore.voice(cue, zh).expect("configured voice cue");
                assert!(!voice.is_empty() && !voice.contains(" · "));
                assert!(lore.lines(zh).any(|line| line.ends_with(voice)));
            }
        }
        assert_eq!(lore.voices.len(), 2);
        for cue in [
            "arrival",
            "transit",
            "awakening",
            "PortalPair",
            "GravityShift",
        ] {
            assert!(lore.voice(cue, false).is_none());
        }
        for hero in 0..HEROES.len() {
            if hero != 12 {
                assert!(role_lore(hero).unwrap().voices.is_empty());
            }
        }
    }
}
