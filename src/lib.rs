//! Physical EN/RU keyboard layout mapping, independent of the Windows input path.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Auto,
    EnToRu,
    RuToEn,
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
        assert_eq!(convert("Ghbdtn, Vbh", Direction::Auto, false), "Приветб Мир");
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
}
