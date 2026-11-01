use std::process::Command;

use inverse_text_normalization_rs::Normalizer;

// Authored examples, not transcripts collected from an ASR system.
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
        let output = Command::new(env!("CARGO_BIN_EXE_normalize"))
            .arg(input)
            .current_dir(std::env::temp_dir())
            .output()?;
        assert!(output.status.success(), "{:?}", output);
        assert_eq!(String::from_utf8(output.stdout)?, format!("{expected}\n"));
        assert!(output.stderr.is_empty());
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
