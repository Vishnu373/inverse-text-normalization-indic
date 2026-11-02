//! Independent approved vocabulary and arithmetic oracle for plan rules 1-14.
use inverse_text_normalization_rs::Normalizer;
use std::sync::OnceLock;

const SMALL: [&str; 20] = [
    "പൂജ്യം",
    "ഒന്ന്",
    "രണ്ട്",
    "മൂന്ന്",
    "നാല്",
    "അഞ്ച്",
    "ആറ്",
    "ഏഴ്",
    "എട്ട്",
    "ഒമ്പത്",
    "പത്ത്",
    "പതിനൊന്ന്",
    "പന്ത്രണ്ട്",
    "പതിമൂന്ന്",
    "പതിനാല്",
    "പതിനഞ്ച്",
    "പതിനാറ്",
    "പതിനേഴ്",
    "പതിനെട്ട്",
    "പത്തൊമ്പത്",
];
const TENS: [&str; 8] = [
    "ഇരുപത്",
    "മുപ്പത്",
    "നാല്പത്",
    "അമ്പത്",
    "അറുപത്",
    "എഴുപത്",
    "എൺപത്",
    "തൊണ്ണൂറ്",
];
const ROOTS: [&str; 8] = [
    "ഇരുപത്ത",
    "മുപ്പത്ത",
    "നാല്പത്ത",
    "അമ്പത്ത",
    "അറുപത്ത",
    "എഴുപത്ത",
    "എൺപത്ത",
    "തൊണ്ണൂറ്റ",
];
const UNITS: [&str; 9] = [
    "ൊന്ന്",
    "ിരണ്ട്",
    "ിമൂന്ന്",
    "ിനാല്",
    "ിയഞ്ച്",
    "ിയാറ്",
    "ിയേഴ്",
    "ിയെട്ട്",
    "ൊമ്പത്",
];
const HUNDREDS: [&str; 9] = [
    "നൂറ്",
    "ഇരുനൂറ്",
    "മുന്നൂറ്",
    "നാനൂറ്",
    "അഞ്ഞൂറ്",
    "അറുനൂറ്",
    "എഴുനൂറ്",
    "എണ്ണൂറ്",
    "തൊള്ളായിരം",
];
const HUNDRED_LINKS: [&str; 9] = [
    "നൂറ്റി",
    "ഇരുനൂറ്റി",
    "മുന്നൂറ്റി",
    "നാനൂറ്റി",
    "അഞ്ഞൂറ്റി",
    "അറുനൂറ്റി",
    "എഴുനൂറ്റി",
    "എണ്ണൂറ്റി",
    "തൊള്ളായിരത്തി",
];
const THOUSAND_PREFIXES: [&str; 20] = [
    "",
    "ആ",
    "രണ്ടാ",
    "മൂവാ",
    "നാലാ",
    "അയ്യാ",
    "ആറാ",
    "ഏഴാ",
    "എണ്ണാ",
    "ഒമ്പതിനാ",
    "പതിനാ",
    "പതിനൊന്നാ",
    "പന്ത്രണ്ടാ",
    "പതിമൂന്നാ",
    "പതിനാലാ",
    "പതിനഞ്ചാ",
    "പതിനാറാ",
    "പതിനേഴാ",
    "പതിനെട്ടാ",
    "പത്തൊമ്പതിനാ",
];
const THOUSAND_TENS: [&str; 8] = [
    "ഇരുപതിനാ",
    "മുപ്പതിനാ",
    "നാല്പതിനാ",
    "അമ്പതിനാ",
    "അറുപതിനാ",
    "എഴുപതിനാ",
    "എൺപതിനാ",
    "തൊണ്ണൂറാ",
];
const THOUSAND_UNITS: [&str; 9] = [
    "ൊന്നാ",
    "ിരണ്ടാ",
    "ിമൂന്നാ",
    "ിനാലാ",
    "ിയഞ്ചാ",
    "ിയാറാ",
    "ിയേഴാ",
    "ിയെട്ടാ",
    "ൊമ്പതിനാ",
];

fn small_word(value: usize) -> String {
    assert!(value < 100);
    if value < 20 {
        return SMALL[value].to_owned();
    }
    let (tens, unit) = (value / 10, value % 10);
    if unit == 0 {
        TENS[tens - 2].to_owned()
    } else {
        format!("{}{}", ROOTS[tens - 2], UNITS[unit - 1])
    }
}

fn thousand_prefix(value: usize) -> String {
    assert!((1..100).contains(&value));
    if value < 20 {
        return THOUSAND_PREFIXES[value].to_owned();
    }
    let (tens, unit) = (value / 10, value % 10);
    if unit == 0 {
        THOUSAND_TENS[tens - 2].to_owned()
    } else {
        format!("{}{}", ROOTS[tens - 2], THOUSAND_UNITS[unit - 1])
    }
}

// Rules 6/8/10/12/14 permit spaced composition, without inferred elisions.
fn spoken(value: usize) -> String {
    assert!(value <= 999_999_999);
    if value < 100 {
        return small_word(value);
    }
    if value < 1000 {
        let (count, rest) = (value / 100, value % 100);
        return if rest == 0 {
            HUNDREDS[count - 1].to_owned()
        } else {
            format!("{} {}", HUNDRED_LINKS[count - 1], small_word(rest))
        };
    }
    if value < 100_000 {
        let (count, rest) = (value / 1000, value % 1000);
        let prefix = thousand_prefix(count);
        return if rest == 0 {
            format!("{prefix}യിരം")
        } else {
            format!("{prefix}യിരത്തി {}", spoken(rest))
        };
    }
    let scale = if value < 10_000_000 {
        100_000
    } else {
        10_000_000
    };
    let (count, rest) = (value / scale, value % scale);
    let head = if count == 1 {
        "ഒരു".to_owned()
    } else {
        small_word(count)
    };
    if scale == 100_000 {
        if rest == 0 {
            format!("{head} ലക്ഷം")
        } else {
            format!("{head} ലക്ഷത്തി {}", spoken(rest))
        }
    } else if rest == 0 {
        format!("{head} കോടി")
    } else {
        format!("{head} കോടി {}", spoken(rest))
    }
}

fn check(text: &str, expected: &str) {
    static NORMALIZER: OnceLock<Normalizer> = OnceLock::new();
    let normalizer = NORMALIZER.get_or_init(|| Normalizer::new("ml").expect("bundled graph"));
    let actual = normalizer.normalize(text).expect("normalization succeeds");
    assert_eq!(actual, expected, "input: {text}");
    assert_eq!(
        normalizer.normalize(&actual).expect("idempotence"),
        actual,
        "input: {text}"
    );
}

#[test]
fn all_81_compound_forms() {
    // 72 canonical compounds and nine using the approved alternate 40-root.
    for (tens, root) in ROOTS
        .iter()
        .enumerate()
        .map(|(i, root)| (i + 2, *root))
        .chain([(4, "നാൽപ്പത്ത")])
    {
        for (unit, suffix) in UNITS.iter().enumerate() {
            check(
                &format!("{root}{suffix}"),
                &(tens * 10 + unit + 1).to_string(),
            );
        }
    }
}

#[test]
fn all_0_to_999() {
    for value in 0..1000 {
        check(&spoken(value), &value.to_string());
    }
}

#[test]
fn spaced_tens_and_spoken_unit_variants() {
    for (tens, root) in ROOTS
        .iter()
        .enumerate()
        .map(|(i, root)| (i + 2, *root))
        .chain([(4, "നാൽപ്പത്ത")])
    {
        for (unit, word) in SMALL[1..10]
            .iter()
            .enumerate()
            .map(|(i, word)| (i + 1, *word))
            .chain([(4, "നാലേ"), (7, "ഏഴേ")])
        {
            for separator in [" ", "\t", "\u{a0}"] {
                check(
                    &format!("{root}ി{separator}{word}"),
                    &(tens * 10 + unit).to_string(),
                );
            }
        }
    }
    check("മുന്നൂറ്റി അമ്പത്തി ഏഴ്", "357");
    check("ഇരുപത്തി നാല് കോടി മുപ്പത്തി ഏഴേ", "240000037");
    check("തൊണ്ണൂറ്റി പൂജ്യം", "തൊണ്ണൂറ്റി 0");
    check("തൊണ്ണൂറ്റി പത്ത്", "തൊണ്ണൂറ്റി 10");
}

#[test]
fn spoken_ending_on_every_whole_number_form() {
    // A spoken final "േ" stands for "്" on any complete whole number.
    let extra = [
        1001,
        2005,
        12_345,
        99_999,
        100_004,
        2_500_004,
        10_000_001,
        999_999_999,
    ];
    for value in (0..1000).chain(extra) {
        let word = spoken(value);
        if let Some(stem) = word.strip_suffix('്') {
            check(&format!("{stem}േ"), &value.to_string());
            check(&format!("ഇവിടെ {stem}േ ഉള്ളൂ"), &format!("ഇവിടെ {value} ഉള്ളൂ"));
        }
    }
    check("മുപ്പത്തിനാലേ", "34");
    check("നൂറ്റി ഒന്നേ", "101");
    check("ഇരുപത്തി അഞ്ച് ലക്ഷത്തി നാലേ", "2500004");
}

#[test]
fn all_thousand_multipliers() {
    for count in 1..100 {
        for rest in [0, 1, 9, 10, 99, 100, 999] {
            let value = count * 1000 + rest;
            check(&spoken(value), &value.to_string());
        }
    }
}

#[test]
fn all_exact_lakh_and_crore_counts() {
    for scale in [100_000, 10_000_000] {
        for count in 1..100 {
            check(&spoken(count * scale), &(count * scale).to_string());
        }
    }
}

#[test]
fn seeded_large_compositions() {
    // Same bounded LCG as Python, independent of Python and all TSVs.
    let mut state = 0x5EED_u32;
    for scale in [100_000, 10_000_000] {
        for _ in 0..256 {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let count = usize::try_from(state % 99 + 1).expect("count fits usize");
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let rest = usize::try_from(state).expect("seed fits usize") % (scale - 1) + 1;
            let value = count * scale + rest;
            check(&spoken(value), &value.to_string());
        }
    }
}

#[test]
fn zero_gaps_and_nine_upper_bounds() {
    // Missing scale chunks must retain their arithmetic place values.
    for crore in [1, 9, 10, 99] {
        for lakh in [0, 1, 99] {
            for thousand in [0, 1, 99] {
                for rest in [0, 1, 9, 99, 100, 999] {
                    let value = crore * 10_000_000 + lakh * 100_000 + thousand * 1000 + rest;
                    check(&spoken(value), &value.to_string());
                }
            }
        }
    }
    for digits in 1..=9 {
        let value = 10_usize.pow(digits) - 1;
        check(&spoken(value), &value.to_string());
    }
}

#[test]
fn repeated_numbers_in_sentences() {
    for value in [
        0, 9, 19, 99, 101, 999, 9000, 19000, 90000, 99000, 100001, 200500, 2500125, 9999999,
        10000001, 20005000, 250000125, 990000000, 999000001, 999999999,
    ] {
        check(
            &format!("എണ്ണം [{}]; വീണ്ടും {}; പൂജ്യം.", spoken(value), spoken(value)),
            &format!("എണ്ണം [{value}]; വീണ്ടും {value}; 0."),
        );
    }
}

#[test]
fn composition_joins_horizontal_whitespace_but_not_line_breaks() {
    for separator in [" ", "\t", "\u{a0}"] {
        check(&format!("ഒരു കോടി{separator}ഒന്ന്"), "10000001");
    }
    for separator in ["\n", "\r\n"] {
        check(
            &format!("ഒരു കോടി{separator}ഒന്ന്"),
            &format!("10000000{separator}1"),
        );
    }
}

#[test]
fn punctuation_unicode_and_identifiers() {
    let value = 990010009;
    for (left, right) in [
        ("[", "]"),
        ("(", ")"),
        ("{", "}"),
        ("\"", "\""),
        ("«", "»"),
        ("₹", "!"),
        ("\u{a0}", "\u{2003}"),
        ("\n", "\t"),
    ] {
        check(
            &format!("{left}{}{right}", spoken(value)),
            &format!("{left}{value}{right}"),
        );
    }
    // Letters, marks, digits, connector punctuation and joiners protect IDs.
    let word = small_word(99);
    for affix in [
        "a", "മ", "7", "_", "\u{203f}", "\u{200c}", "\u{200d}", "\u{301}",
    ] {
        let prefix = format!("{affix}{word}");
        let suffix = format!("{word}{affix}");
        check(&prefix, &prefix);
        check(&suffix, &suffix);
    }
    for text in ["00009", "ID_00009", "൯", "പൂജ്യംx"] {
        check(text, text);
    }
    for separator in [" ", "\t", "\u{a0}"] {
        check(&format!("നൂറ്റി{separator}{}", small_word(25)), "125");
    }
}
