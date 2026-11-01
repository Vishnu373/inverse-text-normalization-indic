use inverse_text_normalization_rs::Normalizer;

#[test]
fn approved_numbers_and_preservation_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    let fixture = include_str!("../wfst/ml/tests/numbers.tsv");
    assert!(!fixture.is_empty(), "empty number fixture");
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
fn preserves_spacing_and_empty_input() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for (spoken, expected) in [
        ("", ""),
        ("  അഞ്ച്\tരണ്ട്\n", "  5\t2\n"),
        ("\u{a0}പൂജ്യം\u{a0}", "\u{a0}0\u{a0}"),
    ] {
        assert_eq!(normalizer.normalize(spoken)?, expected);
    }
    Ok(())
}

#[test]
fn rejects_unsupported_languages() {
    for language in ["en", "hi", "", "ML"] {
        let error = Normalizer::new(language)
            .err()
            .expect("unsupported language");
        assert_eq!(error.code(), "UNSUPPORTED_LANGUAGE");
    }
}

#[test]
fn rejects_nul_instead_of_treating_it_as_epsilon() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    let error = normalizer.normalize("അഞ്ച്\0രണ്ട്").expect_err("NUL input");
    assert_eq!(error.code(), "INVALID_INPUT");
    Ok(())
}
