//! Physical EN/RU keyboard layout mapping, independent of the Windows input path.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Auto,
    EnToRu,
    RuToEn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HotkeySpec {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
    pub key: u32,
    pub label: String,
}

impl HotkeySpec {
    pub fn parse(value: &str) -> Result<Self, String> {
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut win = false;
        let mut key = None;
        let mut key_label = None;

        for raw_part in value.split('+') {
            let part = raw_part.trim().to_ascii_uppercase();
            if part.is_empty() {
                return Err("hotkey contains an empty part".to_string());
            }
            match part.as_str() {
                "CTRL" | "CONTROL" if !ctrl => ctrl = true,
                "ALT" if !alt => alt = true,
                "SHIFT" if !shift => shift = true,
                "WIN" | "WINDOWS" if !win => win = true,
                "CTRL" | "CONTROL" | "ALT" | "SHIFT" | "WIN" | "WINDOWS" => {
                    return Err(format!("duplicate hotkey modifier: {raw_part}"));
                }
                _ if key.is_some() => {
                    return Err("hotkey must contain exactly one regular key".to_string());
                }
                _ => {
                    let parsed = if part.len() == 1 {
                        let ch = part.as_bytes()[0];
                        if ch.is_ascii_uppercase() || ch.is_ascii_digit() {
                            Some(ch as u32)
                        } else {
                            None
                        }
                    } else if let Some(number) = part.strip_prefix('F') {
                        number
                            .parse::<u32>()
                            .ok()
                            .filter(|number| (1..=24).contains(number))
                            .map(|number| 0x70 + number - 1)
                    } else {
                        None
                    };
                    key = Some(parsed.ok_or_else(|| {
                        format!("unsupported hotkey key: {raw_part}; use A-Z, 0-9, or F1-F24")
                    })?);
                    key_label = Some(part);
                }
            }
        }
        if !(ctrl || alt || shift || win) {
            return Err("hotkey must include Ctrl, Alt, Shift, or Win".to_string());
        }
        let key = key.ok_or_else(|| "hotkey is missing its regular key".to_string())?;
        let mut labels = Vec::new();
        if ctrl {
            labels.push("Ctrl".to_string());
        }
        if alt {
            labels.push("Alt".to_string());
        }
        if shift {
            labels.push("Shift".to_string());
        }
        if win {
            labels.push("Win".to_string());
        }
        labels.push(key_label.expect("a parsed key always has a label"));
        Ok(Self {
            ctrl,
            alt,
            shift,
            win,
            key,
            label: labels.join("+"),
        })
    }
}

const EN_LOWER: &str = "`qwertyuiop[]asdfghjkl;'zxcvbnm,./";
const EN_UPPER: &str = "~QWERTYUIOP{}ASDFGHJKL:\"ZXCVBNM<>?";
const EN_SHIFT: &str = "!@#$%^&*()_+";
const RU_LOWER: &str = "ёйцукенгшщзхъфывапролджэячсмитьбю.";
const RU_UPPER: &str = "ЁЙЦУКЕНГШЩЗХЪФЫВАПРОЛДЖЭЯЧСМИТЬБЮ,";
const RU_SHIFT: &str = "!\"№;%:?*()_+";

pub fn convert(input: &str, direction: Direction, lowercase: bool) -> String {
    let direction = match direction {
        Direction::Auto
            if input
                .chars()
                .any(|c| ('\u{0400}'..='\u{04ff}').contains(&c) || c == '№') =>
        {
            Direction::RuToEn
        }
        Direction::Auto => Direction::EnToRu,
        other => other,
    };
    let pairs = [
        (EN_LOWER, RU_LOWER),
        (EN_UPPER, RU_UPPER),
        (EN_SHIFT, RU_SHIFT),
    ];
    let mut mapped = Vec::new();
    for ch in input.chars() {
        let target = pairs.iter().find_map(|(en, ru)| {
            let (source, target) = match direction {
                Direction::EnToRu => (en, ru),
                Direction::RuToEn => (ru, en),
                Direction::Auto => unreachable!(),
            };
            source
                .chars()
                .position(|candidate| candidate == ch)
                .and_then(|index| target.chars().nth(index))
        });
        mapped.push((ch, target.unwrap_or(ch)));
    }
    if lowercase {
        return mapped
            .into_iter()
            .map(|(_, target)| target)
            .collect::<String>()
            .to_lowercase();
    }

    // Punctuation keys can become letters (e.g. ',' -> 'б'). In a CAPS word,
    // give those letters the word's case instead of leaving an accidental lowercase island.
    let mut start = 0;
    while start < mapped.len() {
        if !mapped[start].0.is_alphabetic() && !mapped[start].1.is_alphabetic() {
            start += 1;
            continue;
        }
        let mut end = start;
        let mut has_upper = false;
        let mut has_lower = false;
        while end < mapped.len() && (mapped[end].0.is_alphabetic() || mapped[end].1.is_alphabetic())
        {
            has_upper |= mapped[end].0.is_uppercase();
            has_lower |= mapped[end].0.is_lowercase();
            end += 1;
        }
        if has_upper && !has_lower {
            for (source, target) in &mut mapped[start..end] {
                if !source.is_alphabetic() && target.is_alphabetic() {
                    *target = target.to_uppercase().next().unwrap_or(*target);
                }
            }
        }
        start = end;
    }
    mapped.into_iter().map(|(_, target)| target).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_both_directions() {
        assert_eq!(convert("ghbdtn", Direction::Auto, false), "привет");
        assert_eq!(convert("руддщ", Direction::Auto, false), "hello");
        assert_eq!(
            convert("Ghbdtn? vbh!", Direction::Auto, false),
            "Привет, мир!"
        );
        assert_eq!(
            convert("РУДДЩ, ЦЩКДВ!", Direction::Auto, false),
            "HELLO? WORLD!"
        );
    }

    #[test]
    fn preserves_case_through_punctuation_keys() {
        assert_eq!(convert("Ghbdtn", Direction::Auto, false), "Привет");
        assert_eq!(convert("GHBDTN", Direction::Auto, false), "ПРИВЕТ");
        assert_eq!(convert("HF,JNFTN", Direction::Auto, false), "РАБОТАЕТ");
        assert_eq!(convert("CGFCB,J", Direction::Auto, false), "СПАСИБО");
        assert_eq!(convert("Ghbdtn", Direction::Auto, true), "привет");
        assert_eq!(convert("HF,JNFTN", Direction::Auto, true), "работает");
        assert_eq!(convert("Привет", Direction::Auto, false), "Ghbdtn");
        assert_eq!(convert("ПРИВЕТ", Direction::Auto, false), "GHBDTN");
        assert_eq!(
            convert("Ghbdtn, Vbh", Direction::Auto, false),
            "Приветб Мир"
        );
        assert_eq!(convert("Hf,Jnftn", Direction::Auto, false), "РабОтает");
    }

    #[test]
    fn symbols_and_unmapped_text() {
        assert_eq!(convert("123 ! №", Direction::EnToRu, false), "123 ! №");
        assert_eq!(
            convert("abc\r\ndef", Direction::EnToRu, false),
            "фис\r\nвуа"
        );
    }

    #[test]
    fn parses_supported_hotkeys() {
        assert_eq!(
            HotkeySpec::parse("ctrl+alt+a").unwrap(),
            HotkeySpec {
                ctrl: true,
                alt: true,
                shift: false,
                win: false,
                key: b'A' as u32,
                label: "Ctrl+Alt+A".to_string(),
            }
        );
        assert_eq!(
            HotkeySpec::parse("Shift + Win + F24").unwrap().label,
            "Shift+Win+F24"
        );
        assert_eq!(HotkeySpec::parse("Alt+0").unwrap().key, b'0' as u32);
    }

    #[test]
    fn rejects_unsafe_or_ambiguous_hotkeys() {
        assert!(HotkeySpec::parse("A").is_err());
        assert!(HotkeySpec::parse("Ctrl+Alt").is_err());
        assert!(HotkeySpec::parse("Ctrl+Ctrl+A").is_err());
        assert!(HotkeySpec::parse("Ctrl+A+B").is_err());
        assert!(HotkeySpec::parse("Ctrl+F25").is_err());
        assert!(HotkeySpec::parse("Ctrl+Delete").is_err());
    }
}
