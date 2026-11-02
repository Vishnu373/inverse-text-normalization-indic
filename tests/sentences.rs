use std::process::Command;

use inverse_text_normalization_rs::Normalizer;

// Sentence regressions, not a collected ASR corpus.
const SENTENCES: &[(&str, &str)] = &[
    ("എനിക്ക് നൂറ്റി ഇരുപത്തിയഞ്ച് രൂപ വേണം", "എനിക്ക് 125 രൂപ വേണം"),
    (
        "കൊച്ചിയിലേക്ക് മുപ്പത്തിയേഴ് കിലോമീറ്റർ ദൂരമുണ്ട്.",
        "കൊച്ചിയിലേക്ക് 37 കിലോമീറ്റർ ദൂരമുണ്ട്.",
    ),
    (
        "അരി രണ്ട് കിലോ, പഞ്ചസാര മൂന്ന് കിലോ വേണം.",
        "അരി 2 കിലോ, പഞ്ചസാര 3 കിലോ വേണം.",
    ),
    (
        "പദ്ധതിക്ക് നാല് കോടി ഏഴ് ലക്ഷത്തി അമ്പത്തിരണ്ടായിരത്തി മുന്നൂറ്റി എൺപത്തൊന്ന് രൂപ അനുവദിച്ചു.",
        "പദ്ധതിക്ക് 40752381 രൂപ അനുവദിച്ചു.",
    ),
    ("Order: ഇരുനൂറ്റി അമ്പത്തിയാറ് items.", "Order: 256 items."),
    ("  ബാക്കി\tപൂജ്യം രൂപ.\n", "  ബാക്കി\t0 രൂപ.\n"),
    ("എനിക്ക് 125 രൂപ വേണം.", "എനിക്ക് 125 രൂപ വേണം."),
    ("നാളെ കൊച്ചിയിലേക്ക് പോകാം.", "നാളെ കൊച്ചിയിലേക്ക് പോകാം."),
    ("ഒരു ദിവസം കാണാം.", "ഒരു ദിവസം കാണാം."),
    ("ഇവിടെ മുപ്പത്തി നാലേ ഉള്ളൂ", "ഇവിടെ 34 ഉള്ളൂ"),
    ("ഇവിടെ നാലേ ഉള്ളൂ", "ഇവിടെ 4 ഉള്ളൂ"),
    (
        "കൊച്ചിയിലേക്ക് രണ്ട് പോയിന്റ് ഏഴ് അഞ്ച് കിലോമീറ്റർ ദൂരമുണ്ട്.",
        "കൊച്ചിയിലേക്ക് 2.75 കിലോമീറ്റർ ദൂരമുണ്ട്.",
    ),
    (
        "അരി മൂന്ന് ദശാംശം ഇരുപത്തിയഞ്ച് കിലോ, പഞ്ചസാര പൂജ്യം പോയിന്റ് അഞ്ച് കിലോ വേണം.",
        "അരി 3.25 കിലോ, പഞ്ചസാര 0.5 കിലോ വേണം.",
    ),
    (
        "  അളവ്\tപന്ത്രണ്ട് ദശാംശം പൂജ്യം പൂജ്യം അഞ്ച് പൂജ്യം; വീണ്ടും 12.0050.\n",
        "  അളവ്\t12.0050; വീണ്ടും 12.0050.\n",
    ),
    (
        "നിങ്ങളുടെ ഷുഗർ തൊണ്ണൂറ്റി ഏഴേ ദശാംശം നാലേ അഞ്ച് ആണ്",
        "നിങ്ങളുടെ ഷുഗർ 97.45 ആണ്",
    ),
    (
        "മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ ദശാംശം രണ്ട് ഒന്ന് എട്ട് ഒന്ന് രണ്ട് നാല്",
        "325.218124",
    ),
];

#[test]
fn normalizes_complete_sentences() -> Result<(), Box<dyn std::error::Error>> {
    let normalizer = Normalizer::new("ml")?;
    for &(input, expected) in SENTENCES {
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
fn cli_normalizes_sentences_outside_project_directory() -> Result<(), Box<dyn std::error::Error>> {
    for &(input, expected) in SENTENCES {
        for text in [input, expected] {
            let output = Command::new(env!("CARGO_BIN_EXE_normalize"))
                .arg(text)
                .current_dir(std::env::temp_dir())
                .output()?;
            assert!(output.status.success(), "input: {text}; {output:?}");
            assert_eq!(
                String::from_utf8(output.stdout)?,
                format!("{expected}\n"),
                "input: {text}"
            );
            assert!(output.stderr.is_empty(), "input: {text}");
        }
    }
    Ok(())
}

#[test]
fn cli_requires_exactly_one_argument() -> Result<(), Box<dyn std::error::Error>> {
    for arguments in [vec![], vec!["അഞ്ച്", "രൂപ"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_normalize"))
            .args(arguments)
            .output()?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8(output.stderr)?.contains("Usage:"));
    }
    Ok(())
}

#[test]
fn cli_accepts_empty_input() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_normalize"))
        .arg("")
        .output()?;
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(output.stdout, b"\n");
    assert!(output.stderr.is_empty());
    Ok(())
}

#[cfg(unix)]
#[test]
fn cli_rejects_invalid_utf8_without_panicking() -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let output = Command::new(env!("CARGO_BIN_EXE_normalize"))
        .arg(OsString::from_vec(vec![255]))
        .output()?;
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "Input must be valid UTF-8\n"
    );
    Ok(())
}
