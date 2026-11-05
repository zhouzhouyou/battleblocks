use super::*;

/// Character-file text shown on the final selection page.
pub struct RoleLore {
    title_en: String,
    title_zh: String,
    intro_en: String,
    intro_zh: String,
    traits: Vec<(String, String)>,
    lines: Vec<(String, String)>,
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
