use inverse_text_normalization_rs::Normalizer;

#[test]
fn approved_fraction_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    let fixture = include_str!("../wfst/ml/tests/fractions.tsv");
    assert!(!fixture.is_empty(), "empty fraction fixture");
    for line in fixture.lines() {
        let (spoken, expected) = line.split_once('\t').ok_or("missing fixture delimiter")?;
        if spoken.is_empty() || expected.is_empty() || expected.contains('\t') {
            return Err(format!("invalid fixture: {line}").into());
        }
        let actual = normalizer.normalize(spoken)?;
        assert_eq!(actual, expected, "input: {spoken}");
        assert_eq!(
            normalizer.normalize(&actual)?,
            actual,
            "idempotence: {spoken}"
        );
    }
    Ok(())
}

#[test]
fn common_fractions_and_leg_spelling() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for (input, expected) in [
        ("അര", "1/2"),
        ("കാൽ", "1/4"),
        ("മുക്കാൽ", "3/4"),
        ("കാല്", "കാല്"),
        ("അര കിലോ", "1/2 കിലോ"),
        ("[അര], (കാൽ); മുക്കാൽ! കാല്.", "[1/2], (1/4); 3/4! കാല്."),
    ] {
        let actual = normalizer.normalize(input)?;
        assert_eq!(actual, expected, "input: {input}");
        assert_eq!(
            normalizer.normalize(&actual)?,
            actual,
            "idempotence: {input}"
        );
    }
    Ok(())
}

#[test]
fn productive_mixed_joins_reuse_complete_whole_numbers() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for (input, expected) in [
        ("ഒന്നര", "1 1/2"),
        ("രണ്ടര", "2 1/2"),
        ("അഞ്ചര", "5 1/2"),
        ("പത്തര", "10 1/2"),
        ("ഇരുപത്തഞ്ചര", "25 1/2"),
        ("ഇരുപത്തിയഞ്ചര", "25 1/2"),
        ("നൂറ്റി ഒന്നര", "101 1/2"),
        ("ആയിരത്തി അഞ്ചര", "1005 1/2"),
        ("രണ്ട് ലക്ഷത്തി അഞ്ചര", "200005 1/2"),
        ("ഒരു കോടി ഒന്നര", "10000001 1/2"),
        ("രണ്ടേ കാൽ", "2 1/4"),
        ("രണ്ടേകാൽ", "2 1/4"),
        ("രണ്ടേ മുക്കാൽ", "2 3/4"),
        ("രണ്ടേമുക്കാൽ", "2 3/4"),
        ("ഇരുപത്തിയഞ്ചേകാൽ", "25 1/4"),
        ("നൂറ്റി ഒന്നേ കാൽ", "101 1/4"),
        ("ആയിരത്തി അഞ്ചേമുക്കാൽ", "1005 3/4"),
        ("രണ്ട് ലക്ഷത്തി അഞ്ചേ കാൽ", "200005 1/4"),
        ("ഒരു കോടി ഒന്നേമുക്കാൽ", "10000001 3/4"),
    ] {
        let actual = normalizer.normalize(input)?;
        assert_eq!(actual, expected, "input: {input}");
        assert_eq!(
            normalizer.normalize(&actual)?,
            actual,
            "idempotence: {input}"
        );
    }
    Ok(())
}

#[test]
fn scaled_fractions_have_an_independent_amount_oracle() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    // Independent spoken counts and arithmetic, not production suffix mappings.
    let counts = [
        (1_u64, "ഒന്ന്"),
        (2, "രണ്ട്"),
        (3, "മൂന്ന്"),
        (5, "അഞ്ച്"),
        (10, "പത്ത്"),
        (19, "പത്തൊമ്പത്"),
        (20, "ഇരുപത്"),
        (25, "ഇരുപത്തിയഞ്ച്"),
        (50, "അമ്പത്"),
        (75, "എഴുപത്തിയഞ്ച്"),
        (98, "തൊണ്ണൂറ്റിയെട്ട്"),
        (99, "തൊണ്ണൂറ്റൊമ്പത്"),
    ];
    for (scale_word, scale) in [("ലക്ഷം", 100_000_u64), ("കോടി", 10_000_000)] {
        for (count, word) in counts {
            let stem = word.strip_suffix('്').expect("canonical final virama");
            for (mixed, numerator, denominator) in [
                (format!("{stem}ര"), 1, 2),
                (format!("{stem}േകാൽ"), 1, 4),
                (format!("{stem}േ കാൽ"), 1, 4),
                (format!("{stem}േമുക്കാൽ"), 3, 4),
                (format!("{stem}േ മുക്കാൽ"), 3, 4),
            ] {
                let input = format!("{mixed} {scale_word}");
                let expected =
                    ((count * denominator + numerator) * scale / denominator).to_string();
                let actual = normalizer.normalize(&input)?;
                assert_eq!(actual, expected, "input: {input}");
                assert_eq!(
                    normalizer.normalize(&actual)?,
                    actual,
                    "idempotence: {input}"
                );
            }
        }
        for (fraction, numerator, denominator) in [("അര", 1, 2), ("കാൽ", 1, 4), ("മുക്കാൽ", 3, 4)]
        {
            let input = format!("{fraction} {scale_word}");
            let expected = (numerator * scale / denominator).to_string();
            let actual = normalizer.normalize(&input)?;
            assert_eq!(actual, expected, "input: {input}");
            assert_eq!(
                normalizer.normalize(&actual)?,
                actual,
                "idempotence: {input}"
            );
        }
    }
    Ok(())
}

#[test]
fn scaled_fractions_join_horizontal_whitespace_but_not_lines()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for separator in [" ", "  ", "\t", "\u{a0}", " \t\u{a0} ", "            "] {
        for (input, expected) in [
            (format!("രണ്ടര{separator}ലക്ഷം"), "250000"),
            (format!("ഒന്നേ{separator}കാൽ{separator}ലക്ഷം"), "125000"),
            (format!("മൂന്നേ{separator}മുക്കാൽ{separator}കോടി"), "37500000"),
            (format!("അര{separator}ലക്ഷം"), "50000"),
            (format!("കാൽ{separator}ലക്ഷം"), "25000"),
            (format!("മുക്കാൽ{separator}കോടി"), "7500000"),
        ] {
            let input = format!("\t{input}  ");
            let actual = normalizer.normalize(&input)?;
            assert_eq!(actual, format!("\t{expected}  "), "input: {input:?}");
            assert_eq!(
                normalizer.normalize(&actual)?,
                actual,
                "idempotence: {input:?}"
            );
        }
    }
    for separator in ["\n", "\r\n"] {
        for (input, expected) in [
            (
                format!("രണ്ടര{separator}ലക്ഷം"),
                format!("2 1/2{separator}ലക്ഷം"),
            ),
            (
                format!("ഒന്നേകാൽ{separator}ലക്ഷം"),
                format!("1 1/4{separator}ലക്ഷം"),
            ),
            (
                format!("മൂന്നേ മുക്കാൽ{separator}കോടി"),
                format!("3 3/4{separator}കോടി"),
            ),
            (format!("അര{separator}ലക്ഷം"), format!("1/2{separator}ലക്ഷം")),
            (format!("കാൽ{separator}ലക്ഷം"), format!("1/4{separator}ലക്ഷം")),
            (
                format!("മുക്കാൽ{separator}കോടി"),
                format!("3/4{separator}കോടി"),
            ),
            (
                format!("ഒന്നേ{separator}കാൽ ലക്ഷം"),
                format!("1{separator}25000"),
            ),
            (
                format!("മൂന്നേ{separator}മുക്കാൽ കോടി"),
                format!("3{separator}7500000"),
            ),
        ] {
            let actual = normalizer.normalize(&input)?;
            assert_eq!(actual, expected, "input: {input:?}");
            assert_eq!(
                normalizer.normalize(&actual)?,
                actual,
                "idempotence: {input:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn scaled_fractions_keep_boundaries_and_existing_operand_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for (word, scale, unscaled) in [
        ("രണ്ടര", "ലക്ഷം", "2 1/2"),
        ("അര", "ലക്ഷം", "1/2"),
        ("മുക്കാൽ", "കോടി", "3/4"),
    ] {
        for affix in [
            "a", "മ", "7", "_", "\u{203f}", "\u{200c}", "\u{200d}", "\u{301}",
        ] {
            for (input, expected) in [
                (
                    format!("{affix}{word} {scale}"),
                    format!("{affix}{word} {scale}"),
                ),
                (
                    format!("{word} {scale}{affix}"),
                    format!("{unscaled} {scale}{affix}"),
                ),
            ] {
                let actual = normalizer.normalize(&input)?;
                assert_eq!(actual, expected, "input: {input:?}");
                assert_eq!(
                    normalizer.normalize(&actual)?,
                    actual,
                    "idempotence: {input:?}"
                );
            }
        }
    }
    for (input, expected) in [
        (
            "[രണ്ടര ലക്ഷം], (ഒന്നേകാൽ ലക്ഷം); മൂന്നേ മുക്കാൽ കോടി!",
            "[250000], (125000); 37500000!",
        ),
        ("രണ്ടരലക്ഷം", "രണ്ടരലക്ഷം"),
        ("അരലക്ഷം", "അരലക്ഷം"),
        ("രണ്ടര, ലക്ഷം", "2 1/2, ലക്ഷം"),
        ("അര, ലക്ഷം", "1/2, ലക്ഷം"),
        ("നൂറര ലക്ഷം", "100 1/2 ലക്ഷം"),
        ("നൂറേകാൽ കോടി", "100 1/4 കോടി"),
        ("രണ്ടര ലക്ഷം ബാർ മൂന്ന്", "250000 ബാർ 3"),
        ("മൂന്ന് ബാർ അര ലക്ഷം", "3 ബാർ 50000"),
        ("രണ്ടര ലക്ഷം പോയിന്റ് അഞ്ച്", "250000 പോയിന്റ് 5"),
        ("രണ്ട് പോയിന്റ് അര ലക്ഷം", "2 പോയിന്റ് 50000"),
    ] {
        let actual = normalizer.normalize(input)?;
        assert_eq!(actual, expected, "input: {input}");
        assert_eq!(
            normalizer.normalize(&actual)?,
            actual,
            "idempotence: {input}"
        );
    }
    Ok(())
}

#[test]
fn plain_adjacency_preserves_common_fraction_words() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for (input, expected) in [
        ("അഞ്ച് കാൽ മേശ", "5 കാൽ മേശ"),
        ("രണ്ടേ അര", "2 അര"),
        ("5 കാൽ", "5 കാൽ"),
        ("1000 അര", "1000 അര"),
        ("0 അര", "0 അര"),
    ] {
        let actual = normalizer.normalize(input)?;
        assert_eq!(actual, expected, "input: {input}");
        assert_eq!(
            normalizer.normalize(&actual)?,
            actual,
            "idempotence: {input}"
        );
    }
    for (word, value) in [
        ("രണ്ട്", "2"),
        ("അഞ്ച്", "5"),
        ("പൂജ്യം", "0"),
        ("ആയിരം", "1000"),
        ("ഒരു ലക്ഷം", "100000"),
        ("ഒരു കോടി", "10000000"),
        ("തൊള്ളായിരം", "900"),
        ("നൂറ്റി ഒന്ന്", "101"),
        ("രണ്ട് ലക്ഷത്തി അഞ്ച്", "200005"),
    ] {
        for fraction in ["അര", "കാൽ", "മുക്കാൽ"] {
            for separator in [" ", "  ", "\t", "\u{a0}", " \t\u{a0} "] {
                let input = format!("\t{word}{separator}{fraction}  ");
                let actual = normalizer.normalize(&input)?;
                assert_eq!(
                    actual,
                    format!("\t{value}{separator}{fraction}  "),
                    "input: {input:?}"
                );
                assert_eq!(
                    normalizer.normalize(&actual)?,
                    actual,
                    "idempotence: {input:?}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn adjacency_guard_does_not_displace_compound_numbers() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for separator in [
        " ".repeat(5),
        " ".repeat(6),
        " ".repeat(10),
        " ".repeat(11),
        "\t".repeat(8),
        "\u{a0}".repeat(8),
    ] {
        for (text, expected) in [
            ("രണ്ട് ബാർ അഞ്ച്", "2/5"),
            ("അഞ്ച് പോയിന്റ് രണ്ട്", "5.2"),
            ("നാലിൽ മൂന്ന്", "3/4"),
        ] {
            let input = format!("{text}{separator}അര");
            let actual = normalizer.normalize(&input)?;
            assert_eq!(
                actual,
                format!("{expected}{separator}1/2"),
                "input: {input:?}"
            );
            assert_eq!(normalizer.normalize(&actual)?, actual);
        }
        let input = format!("നൂറ്റി{separator}അഞ്ച്{separator}കാൽ");
        let actual = normalizer.normalize(&input)?;
        assert_eq!(actual, format!("105{separator}കാൽ"), "input: {input:?}");
        assert_eq!(normalizer.normalize(&actual)?, actual);
    }
    Ok(())
}

#[test]
fn bar_operands_cover_the_existing_number_range_without_reduction()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    let numbers = [
        ("പൂജ്യം", 0),
        ("ഒന്ന്", 1),
        ("രണ്ട്", 2),
        ("മൂന്ന്", 3),
        ("നാല്", 4),
        ("അഞ്ച്", 5),
        ("ഏഴ്", 7),
        ("തൊണ്ണൂറ്റൊമ്പത്", 99),
        ("നൂറ്", 100),
        ("ആയിരത്തി ഒന്ന്", 1001),
        ("രണ്ട് ലക്ഷത്തി അഞ്ച്", 200005),
        ("ഒരു കോടി ഒന്ന്", 10000001),
        (
            "തൊണ്ണൂറ്റൊമ്പത് കോടി തൊണ്ണൂറ്റൊമ്പത് ലക്ഷത്തി തൊണ്ണൂറ്റൊമ്പതിനായിരത്തി തൊള്ളായിരത്തി തൊണ്ണൂറ്റൊമ്പത്",
            999999999,
        ),
    ];
    for (numerator, n) in numbers {
        for (denominator, d) in numbers {
            if d == 0 {
                continue;
            }
            let input = format!("{numerator} ബാർ {denominator}");
            let actual = normalizer.normalize(&input)?;
            assert_eq!(actual, format!("{n}/{d}"), "input: {input}");
            assert_eq!(
                normalizer.normalize(&actual)?,
                actual,
                "idempotence: {input}"
            );
        }
        let input = format!("നൂറിൽ {numerator}");
        let actual = normalizer.normalize(&input)?;
        assert_eq!(actual, format!("{n}/100"), "input: {input}");
        assert_eq!(
            normalizer.normalize(&actual)?,
            actual,
            "idempotence: {input}"
        );
    }
    Ok(())
}

#[test]
fn every_denominator_from_one_through_one_hundred_has_an_independent_oracle()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    // Independent vocabulary, not production mappings or another test's generator.
    let small = [
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
    let tens = [
        "ഇരുപത്",
        "മുപ്പത്",
        "നാല്പത്",
        "അമ്പത്",
        "അറുപത്",
        "എഴുപത്",
        "എൺപത്",
        "തൊണ്ണൂറ്",
    ];
    let roots = [
        "ഇരുപത്ത",
        "മുപ്പത്ത",
        "നാല്പത്ത",
        "അമ്പത്ത",
        "അറുപത്ത",
        "എഴുപത്ത",
        "എൺപത്ത",
        "തൊണ്ണൂറ്റ",
    ];
    let units = [
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
    for denominator in 1..=100 {
        let word = if denominator < 20 {
            small[denominator - 1].to_owned()
        } else if denominator == 100 {
            "നൂറ്".to_owned()
        } else if denominator % 10 == 0 {
            tens[denominator / 10 - 2].to_owned()
        } else {
            format!(
                "{}{}",
                roots[denominator / 10 - 2],
                units[denominator % 10 - 1]
            )
        };
        let stem = word.strip_suffix('്').expect("canonical final virama");
        for (input, expected) in [
            (format!("{stem}ിൽ മൂന്ന്"), format!("3/{denominator}")),
            (format!("{stem}ിലൊന്ന്"), format!("1/{denominator}")),
        ] {
            let actual = normalizer.normalize(&input)?;
            assert_eq!(actual, expected, "input: {input}");
            assert_eq!(
                normalizer.normalize(&actual)?,
                actual,
                "idempotence: {input}"
            );
        }
    }
    for (input, expected) in [
        ("രണ്ടിൽ ഒന്ന്", "1/2"),
        ("മൂന്നിൽ ഒന്ന്", "1/3"),
        ("നാലിൽ മൂന്ന്", "3/4"),
        ("നാലിൽ രണ്ട്", "2/4"),
        ("അഞ്ചിൽ പൂജ്യം", "0/5"),
        ("പത്തിൽ അഞ്ച്", "5/10"),
        ("ഇരുപതിൽ അഞ്ച്", "5/20"),
        ("അമ്പതിൽ അഞ്ച്", "5/50"),
        ("നൂറിൽ മൂന്ന്", "3/100"),
        ("ഇരുപത്തി അഞ്ചിൽ അഞ്ച്", "5/25"),
        ("നാൽപ്പതിൽ രണ്ട്", "2/40"),
        ("ഇരുപത്തി അഞ്ചിലൊന്ന്", "1/25"),
        ("ഒന്‍പതിലൊന്ന്", "1/9"),
    ] {
        let actual = normalizer.normalize(input)?;
        assert_eq!(actual, expected, "input: {input}");
        assert_eq!(
            normalizer.normalize(&actual)?,
            actual,
            "idempotence: {input}"
        );
    }
    Ok(())
}

#[test]
fn spoken_endings_remain_valid_on_complete_fraction_components()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for (word, value) in [
        ("ഒന്നേ", "1"),
        ("നാലേ", "4"),
        ("ഏഴേ", "7"),
        ("പതിനഞ്ചേ", "15"),
        ("ഇരുപത്തിയഞ്ചേ", "25"),
        ("നൂറേ", "100"),
        ("മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "325"),
        ("ആയിരത്തി മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "1325"),
        ("രണ്ട് ലക്ഷത്തി മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "200325"),
        ("ഒരു കോടി മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "10000325"),
    ] {
        for (input, expected) in [
            (format!("{word} ബാർ {word}"), format!("{value}/{value}")),
            (format!("നാലിൽ {word}"), format!("{value}/4")),
            (format!("{word} അര"), format!("{value} അര")),
            (format!("{word} കാൽ"), format!("{value} 1/4")),
            (format!("{word} മുക്കാൽ"), format!("{value} 3/4")),
        ] {
            let actual = normalizer.normalize(&input)?;
            assert_eq!(actual, expected, "input: {input}");
            assert_eq!(
                normalizer.normalize(&actual)?,
                actual,
                "idempotence: {input}"
            );
        }
    }
    Ok(())
}

#[test]
fn joins_horizontal_spacing_but_not_line_breaks() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for separator in [" ", "  ", "\t", "\u{a0}", " \t\u{a0} "] {
        for (input, expected) in [
            (format!("രണ്ട്{separator}ബാർ{separator}മൂന്ന്"), "2/3"),
            (format!("നാലിൽ{separator}രണ്ട്"), "2/4"),
            (format!("രണ്ടേ{separator}കാൽ"), "2 1/4"),
            (format!("രണ്ടേ{separator}മുക്കാൽ"), "2 3/4"),
            (
                format!("നൂറ്റി{separator}ഒന്ന്{separator}ബാർ{separator}മൂന്ന്"),
                "101/3",
            ),
        ] {
            let input = format!("\t{input}  ");
            let actual = normalizer.normalize(&input)?;
            assert_eq!(actual, format!("\t{expected}  "), "input: {input:?}");
            assert_eq!(
                normalizer.normalize(&actual)?,
                actual,
                "idempotence: {input:?}"
            );
        }
    }
    for separator in ["\n", "\r\n"] {
        for (input, expected) in [
            (format!("രണ്ട്{separator}ബാർ മൂന്ന്"), format!("2{separator}ബാർ 3")),
            (format!("രണ്ട് ബാർ{separator}മൂന്ന്"), format!("2 ബാർ{separator}3")),
            (format!("നാലിൽ{separator}രണ്ട്"), format!("നാലിൽ{separator}2")),
            (format!("രണ്ടേ{separator}കാൽ"), format!("2{separator}1/4")),
            (format!("രണ്ടേ{separator}മുക്കാൽ"), format!("2{separator}3/4")),
            (format!("പൂജ്യം{separator}അര"), format!("0{separator}1/2")),
        ] {
            let actual = normalizer.normalize(&input)?;
            assert_eq!(actual, expected, "input: {input:?}");
            assert_eq!(
                normalizer.normalize(&actual)?,
                actual,
                "idempotence: {input:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn respects_unicode_word_boundaries_and_preserves_identifiers()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for word in [
        "അര",
        "കാൽ",
        "മുക്കാൽ",
        "ഒന്നര",
        "രണ്ടേകാൽ",
        "രണ്ടേമുക്കാൽ",
        "മൂന്നിലൊന്ന്",
        "അഞ്ചിലൊന്ന്",
    ] {
        for affix in [
            "a", "മ", "7", "_", "\u{203f}", "\u{200c}", "\u{200d}", "\u{301}",
        ] {
            for input in [format!("{affix}{word}"), format!("{word}{affix}")] {
                assert_eq!(normalizer.normalize(&input)?, input, "input: {input:?}");
            }
        }
    }
    for (input, expected) in [
        ("[രണ്ട് ബാർ നാല്], (നാലിൽ മൂന്ന്)!", "[2/4], (3/4)!"),
        ("[മൂന്നിലൊന്ന്], (അഞ്ചിലൊന്ന്)!", "[1/3], (1/5)!"),
        ("{ഒന്നര}; \"രണ്ടേകാൽ\"!", "{1 1/2}; \"2 1/4\"!"),
        ("രണ്ട്, ബാർ മൂന്ന്", "2, ബാർ 3"),
        ("രണ്ട് ബാർ, മൂന്ന്", "2 ബാർ, 3"),
        ("നാലിൽ, രണ്ട്", "നാലിൽ, 2"),
        ("രണ്ടേ, കാൽ", "2, 1/4"),
        ("id_രണ്ട് ബാർ മൂന്ന്", "id_രണ്ട് ബാർ 3"),
        ("രണ്ട് ബാർ മൂന്ന്_id", "2 ബാർ മൂന്ന്_id"),
        ("രണ്ട് ബാർ_id മൂന്ന്", "2 ബാർ_id 3"),
        ("id_നാലിൽ രണ്ട്", "id_നാലിൽ 2"),
        ("നാലിൽ രണ്ട്_id", "നാലിൽ രണ്ട്_id"),
        ("id_രണ്ടേ കാൽ", "id_രണ്ടേ 1/4"),
        ("രണ്ടേ കാൽ_id", "2 കാൽ_id"),
        ("id_രണ്ട്_ബാർ_മൂന്ന്", "id_രണ്ട്_ബാർ_മൂന്ന്"),
        ("അരിവാൾ കാലിൽ തട്ടി.", "അരിവാൾ കാലിൽ തട്ടി."),
        ("കാല്", "കാല്"),
    ] {
        let actual = normalizer.normalize(input)?;
        assert_eq!(actual, expected, "input: {input}");
        assert_eq!(
            normalizer.normalize(&actual)?,
            actual,
            "idempotence: {input}"
        );
    }
    Ok(())
}

#[test]
fn unsupported_forms_keep_existing_partial_rewriting() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for (input, expected) in [
        ("രണ്ട് bar മൂന്ന്", "2 bar 3"),
        ("രണ്ട് by മൂന്ന്", "2 by 3"),
        ("രണ്ട് ബൈ മൂന്ന്", "2 ബൈ 3"),
        ("ബാർ", "ബാർ"),
        ("രണ്ട് ബാർ", "2 ബാർ"),
        ("ബാർ മൂന്ന്", "ബാർ 3"),
        ("രണ്ട്ബാർമൂന്ന്", "രണ്ട്ബാർമൂന്ന്"),
        ("രണ്ട്ബാർ മൂന്ന്", "രണ്ട്ബാർ 3"),
        ("രണ്ട് ബാർമൂന്ന്", "2 ബാർമൂന്ന്"),
        ("രണ്ട് ബാർ പൂജ്യം", "2 ബാർ 0"),
        ("പൂജ്യം ബാർ പൂജ്യം", "0 ബാർ 0"),
        ("2 ബാർ മൂന്ന്", "2 ബാർ 3"),
        ("രണ്ട് ബാർ 3", "2 ബാർ 3"),
        ("നാലിൽ 2", "നാലിൽ 2"),
        ("നാലിൽരണ്ട്", "നാലിൽരണ്ട്"),
        ("4ൽ രണ്ട്", "4ൽ 2"),
        ("നൂറ്റൊന്നിൽ രണ്ട്", "നൂറ്റൊന്നിൽ 2"),
        ("ആയിരത്തിൽ ഒന്ന്", "ആയിരത്തിൽ 1"),
        ("പൂജ്യത്തിൽ ഒന്ന്", "പൂജ്യത്തിൽ 1"),
        ("നൂറ്റൊന്നിലൊന്ന്", "നൂറ്റൊന്നിലൊന്ന്"),
        ("ആയിരത്തിലൊന്ന്", "ആയിരത്തിലൊന്ന്"),
        ("പൂജ്യത്തിലൊന്ന്", "പൂജ്യത്തിലൊന്ന്"),
        ("മൂന്നിലിരണ്ട്", "മൂന്നിലിരണ്ട്"),
        ("നാലിലൊന്ന് ബാർ മൂന്ന്", "1/4 ബാർ 3"),
        ("നാലിൽ", "നാലിൽ"),
        ("നാല് ഭാഗം മൂന്ന്", "4 ഭാഗം 3"),
        ("നാലിൽ മൂന്ന് ഭാഗം", "3/4 ഭാഗം"),
        ("അര ബാർ മൂന്ന്", "1/2 ബാർ 3"),
        ("രണ്ട് ബാർ അര", "2 ബാർ 1/2"),
        ("രണ്ട്  മൂന്ന് ബാർ നാല്", "2  3/4"),
        ("രണ്ട്\tമൂന്ന് ബാർ നാല്", "2\t3/4"),
        ("മൈനസ് രണ്ട് ബാർ മൂന്ന്", "മൈനസ് 2/3"),
        ("-രണ്ട് ബാർ മൂന്ന്", "-2/3"),
        ("", ""),
        (
            "  0002/0004\t0/7 5/3 2/4 0 1/2\r\n",
            "  0002/0004\t0/7 5/3 2/4 0 1/2\r\n",
        ),
        ("ID_2/4", "ID_2/4"),
    ] {
        let actual = normalizer.normalize(input)?;
        assert_eq!(actual, expected, "input: {input:?}");
        assert_eq!(
            normalizer.normalize(&actual)?,
            actual,
            "idempotence: {input:?}"
        );
    }
    Ok(())
}
