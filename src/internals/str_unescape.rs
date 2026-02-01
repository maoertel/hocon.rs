use aho_corasick::AhoCorasick;
use std::borrow::Cow;
use std::ops::Range;
use std::sync::OnceLock;

const PATTERNS: &[&str] = &[
    r#"\""#, r"\\", r"\/", r"\b", r"\f", r"\n", r"\r", r"\t", r"\u",
];
const REPLACEMENTS: &[&str] = &["\"", "\\", "/", "\x08", "\x0c", "\x0a", "\x0d", "\x09"];

fn automaton() -> &'static AhoCorasick {
    static AC: OnceLock<AhoCorasick> = OnceLock::new();
    AC.get_or_init(|| {
        // SAFETY: patterns are hardcoded valid strings, this cannot fail
        AhoCorasick::new(PATTERNS).unwrap()
    })
}

/// Unescape a JSON string
pub(crate) fn unescape(input: &str) -> Cow<'_, str> {
    const HIGH_SURROGATES: Range<u16> = 0xd800..0xdc00;
    const LOW_SURROGATES: Range<u16> = 0xdc00..0xe000;

    let mut res = Cow::default();
    let mut last_start: usize = 0;
    let mut surrogates_vec: [u16; 2] = [0, 0];
    for mat in automaton().find_iter(input) {
        res += &input[last_start..mat.start()];
        last_start = mat.end();

        if let Some(repl) = REPLACEMENTS.get(mat.pattern().as_usize()) {
            res += *repl;
        } else if mat.end() + 4 <= input.len() {
            // Handle \u
            last_start += 4;
            let hex_digits = &input[mat.end()..mat.end() + 4];
            if let Ok(cp) = u16::from_str_radix(hex_digits, 16) {
                // Handle Unicode surrogate pairs
                if HIGH_SURROGATES.contains(&cp) {
                    // Beginning of surrogate pair
                    surrogates_vec[0] = cp;
                } else {
                    surrogates_vec[1] = cp;
                    let surrogates_vec_ref = if LOW_SURROGATES.contains(&cp) {
                        // Ending of surrogate pair, call: from_utf16([high, low])
                        &surrogates_vec
                    } else {
                        // Not a surrogate pair, call: from_utf16([cp])
                        &surrogates_vec[1..]
                    };
                    if let Ok(str) = String::from_utf16(surrogates_vec_ref) {
                        res += Cow::from(str);
                    }
                }
            }
        }
    }
    res += &input[last_start..];
    res
}
