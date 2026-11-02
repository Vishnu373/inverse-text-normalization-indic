use inverse_text_normalization_rs::Normalizer;

#[test]
fn approved_decimal_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    let fixture = include_str!("../wfst/ml/tests/decimals.tsv");
    assert!(!fixture.is_empty(), "empty decimal fixture");
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
fn spoken_endings_reuse_number_rules_in_decimal_components()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for marker in ["പോയിന്റ്", "ദശാംശം"] {
        for (word, value) in [
            ("ഒന്നേ", "1"),
            ("രണ്ടേ", "2"),
            ("മൂന്നേ", "3"),
            ("നാലേ", "4"),
            ("അഞ്ചേ", "5"),
            ("ആറേ", "6"),
            ("ഏഴേ", "7"),
            ("എട്ടേ", "8"),
            ("ഒമ്പതേ", "9"),
            ("പതിനഞ്ചേ", "15"),
            ("ഇരുപത്തിയഞ്ചേ", "25"),
            ("തൊണ്ണൂറ്റൊമ്പതേ", "99"),
            ("നൂറേ", "100"),
            ("മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "325"),
            ("ആയിരത്തി മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "1325"),
            ("രണ്ട് ലക്ഷത്തി മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "200325"),
            ("ഒരു കോടി മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ", "10000325"),
        ] {
            for (input, expected) in [
                (
                    format!("{word} {marker} രണ്ട് ഒന്ന് എട്ട് ഒന്ന് രണ്ട് നാല്"),
                    format!("{value}.218124"),
                ),
                (format!("രണ്ട് {marker} {word}"), format!("2.{value}")),
            ] {
                let actual = normalizer.normalize(&input)?;
                assert_eq!(actual, expected, "input: {input}");
                assert_eq!(normalizer.normalize(&actual)?, actual, "input: {input}");
            }
            if !word.contains(' ') {
                for (input, expected) in [
                    (word.to_owned(), value.to_owned()),
                    (
                        format!("{word}x {marker} അഞ്ച്"),
                        format!("{word}x {marker} 5"),
                    ),
                    (
                        format!("രണ്ട് {marker} {word}_id"),
                        format!("2 {marker} {word}_id"),
                    ),
                ] {
                    assert_eq!(normalizer.normalize(&input)?, expected, "input: {input}");
                }
            }
            if value.len() == 1 {
                let input = format!("പന്ത്രണ്ട് {marker} പൂജ്യം {word} പൂജ്യം");
                let expected = format!("12.0{value}0");
                assert_eq!(normalizer.normalize(&input)?, expected, "input: {input}");
            }
        }
    }
    Ok(())
}

#[test]
fn both_markers_accept_cardinals_and_preserve_fractional_digits()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for marker in ["പോയിന്റ്", "ദശാംശം"] {
        for (integer, fraction, expected) in [
            ("പൂജ്യം", "അഞ്ച്", "0.5"),
            ("രണ്ട്", "പൂജ്യം", "2.0"),
            ("ഇരുപത്തിമൂന്ന്", "നാല്പത്തിയഞ്ച്", "23.45"),
            ("നൂറ്റി ഒന്ന്", "ഇരുനൂറ്റി മൂന്ന്", "101.203"),
            ("രണ്ടായിരത്തി അഞ്ച്", "ആയിരത്തി ഒന്ന്", "2005.1001"),
            ("രണ്ട് ലക്ഷം", "മൂന്ന് ലക്ഷം", "200000.300000"),
            ("ഒരു കോടി ഒന്ന്", "രണ്ട് കോടി അഞ്ച്", "10000001.20000005"),
            ("രണ്ട്", "പൂജ്യം പൂജ്യം അഞ്ച്", "2.005"),
            ("രണ്ട്", "അഞ്ച് പൂജ്യം", "2.50"),
            ("പന്ത്രണ്ട്", "പൂജ്യം പൂജ്യം അഞ്ച് പൂജ്യം", "12.0050"),
            ("പൂജ്യം", "പൂജ്യം പൂജ്യം പൂജ്യം", "0.000"),
            (
                "രണ്ട്",
                "ഒന്ന് രണ്ട് മൂന്ന് നാല് അഞ്ച് ആറ് ഏഴ് എട്ട് ഒമ്പത് പൂജ്യം",
                "2.1234567890",
            ),
        ] {
            let input = format!("{integer} {marker} {fraction}");
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
    for (input, expected) in [
        ("  രണ്ട്  പോയിന്റ്   അഞ്ച് പൂജ്യം  ", "  2.50  "),
        ("\tരണ്ട്\tപോയിന്റ്\tഅഞ്ച്\tപൂജ്യം\t", "\t2.50\t"),
        ("രണ്ട്\u{a0}പോയിന്റ്\u{a0}അഞ്ച്\u{a0}പൂജ്യം", "2.50"),
        ("രണ്ട്  ദശാംശം   അഞ്ച് പൂജ്യം", "2.50"),
        ("രണ്ട്\tദശാംശം\tഅഞ്ച്\tപൂജ്യം", "2.50"),
        ("രണ്ട്\u{a0}ദശാംശം\u{a0}അഞ്ച്\u{a0}പൂജ്യം", "2.50"),
        ("നൂറ്റി\tഒന്ന്\u{a0}പോയിന്റ്\tഇരുനൂറ്റി\u{a0}മൂന്ന്", "101.203"),
        ("രണ്ട് പോയിന്റ്\nഅഞ്ച്", "2 പോയിന്റ്\n5"),
        ("രണ്ട് പോയിന്റ്\r\nഅഞ്ച്", "2 പോയിന്റ്\r\n5"),
        ("രണ്ട് ദശാംശം\nഅഞ്ച്", "2 ദശാംശം\n5"),
        ("രണ്ട് ദശാംശം\r\nഅഞ്ച്", "2 ദശാംശം\r\n5"),
        ("രണ്ട്\nപോയിന്റ് അഞ്ച്", "2\nപോയിന്റ് 5"),
        ("രണ്ട്\r\nപോയിന്റ് അഞ്ച്", "2\r\nപോയിന്റ് 5"),
        ("രണ്ട്\nദശാംശം അഞ്ച്", "2\nദശാംശം 5"),
        ("രണ്ട്\r\nദശാംശം അഞ്ച്", "2\r\nദശാംശം 5"),
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

#[test]
fn spoken_digit_variants_compose_and_respect_word_boundaries()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for marker in ["പോയിന്റ്", "ദശാംശം"] {
        for (integer, integer_digits) in [
            ("പന്ത്രണ്ട്", "12"),
            ("നാലേ", "4"),
            ("ഏഴേ", "7"),
            ("മുപ്പത്തി നാലേ", "34"),
            ("നൂറ്റി തൊണ്ണൂറ്റി ഏഴേ", "197"),
        ] {
            for (fraction, fraction_digits) in [
                ("നാലേ അഞ്ച്", "45"),
                ("ഏഴേ നാലേ", "74"),
                ("പൂജ്യം നാലേ പൂജ്യം", "040"),
                ("നാലേ", "4"),
                ("അറുപത്തി ഏഴേ", "67"),
            ] {
                let input = format!("{integer} {marker} {fraction}");
                let expected = format!("{integer_digits}.{fraction_digits}");
                let actual = normalizer.normalize(&input)?;
                assert_eq!(actual, expected, "input: {input}");
                assert_eq!(normalizer.normalize(&actual)?, actual, "input: {input}");
            }
        }
    }
    for (input, expected) in [
        ("ഇവിടെ നാലേ ഉള്ളൂ", "ഇവിടെ 4 ഉള്ളൂ"),
        ("ഇവിടെ ഏഴേ ഉള്ളൂ", "ഇവിടെ 7 ഉള്ളൂ"),
        ("രണ്ട് ദശാംശം നാലേ_id", "2 ദശാംശം നാലേ_id"),
        ("id_ഏഴേ ദശാംശം അഞ്ച്", "id_ഏഴേ ദശാംശം 5"),
        ("തൊണ്ണൂറ്റി\nഏഴേ ദശാംശം നാലേ അഞ്ച്", "തൊണ്ണൂറ്റി\n7.45"),
        ("തൊണ്ണൂറ്റി ഏഴേ ദശാംശം\nനാലേ അഞ്ച്", "97 ദശാംശം\n4 5"),
    ] {
        let actual = normalizer.normalize(input)?;
        assert_eq!(actual, expected, "input: {input}");
        assert_eq!(normalizer.normalize(&actual)?, actual, "input: {input}");
    }
    Ok(())
}

#[test]
fn respects_punctuation_identifiers_and_incomplete_phrases()
-> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for (input, expected) in [
        ("[രണ്ട് പോയിന്റ് അഞ്ച്], (മൂന്ന് ദശാംശം പൂജ്യം ആറ്)!", "[2.5], (3.06)!"),
        ("രണ്ട്, പോയിന്റ് അഞ്ച്", "2, പോയിന്റ് 5"),
        ("രണ്ട് പോയിന്റ്, അഞ്ച്", "2 പോയിന്റ്, 5"),
        ("രണ്ട്, ദശാംശം അഞ്ച്", "2, ദശാംശം 5"),
        ("രണ്ട് ദശാംശം, അഞ്ച്", "2 ദശാംശം, 5"),
        ("id_രണ്ട് പോയിന്റ് അഞ്ച്", "id_രണ്ട് പോയിന്റ് 5"),
        ("രണ്ട് പോയിന്റ് അഞ്ച്_id", "2 പോയിന്റ് അഞ്ച്_id"),
        ("രണ്ട് പോയിന്റ്_tag അഞ്ച്", "2 പോയിന്റ്_tag 5"),
        ("id_രണ്ട് ദശാംശം അഞ്ച്", "id_രണ്ട് ദശാംശം 5"),
        ("രണ്ട് ദശാംശം അഞ്ച്_id", "2 ദശാംശം അഞ്ച്_id"),
        ("രണ്ട് ദശാംശം_tag അഞ്ച്", "2 ദശാംശം_tag 5"),
        ("id_രണ്ട്_പോയിന്റ്_അഞ്ച്", "id_രണ്ട്_പോയിന്റ്_അഞ്ച്"),
        ("id_രണ്ട്_ദശാംശം_അഞ്ച്", "id_രണ്ട്_ദശാംശം_അഞ്ച്"),
        ("പോയിന്റ്", "പോയിന്റ്"),
        ("ദശാംശം", "ദശാംശം"),
        ("രണ്ട് പോയിന്റ്", "2 പോയിന്റ്"),
        ("രണ്ട് ദശാംശം", "2 ദശാംശം"),
        ("പോയിന്റ് അഞ്ച്", "പോയിന്റ് 5"),
        ("ദശാംശം അഞ്ച്", "ദശാംശം 5"),
        ("", ""),
        (
            "  0002.0050\t0.000 12.50\r\n",
            "  0002.0050\t0.000 12.50\r\n",
        ),
        ("ID_0002.0050", "ID_0002.0050"),
        ("നാളെ കൊച്ചിയിലേക്ക് പോകാം.", "നാളെ കൊച്ചിയിലേക്ക് പോകാം."),
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
