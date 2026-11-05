//! WFST inverse text normalization for Malayalam whole numbers, decimals, and fractions.

mod runtime;

use std::fmt;

use runtime::Runtime;

/// A normalization failure with a stable machine-readable code.
#[derive(Debug)]
pub enum AppError {
    UnsupportedLanguage(String),
    InvalidInput(&'static str),
    GraphLoad(String),
    GraphExecution(String),
}

impl AppError {
    /// Return the stable error code, independent of the diagnostic message.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedLanguage(_) => "UNSUPPORTED_LANGUAGE",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::GraphLoad(_) => "GRAPH_LOAD_FAILED",
            Self::GraphExecution(_) => "GRAPH_EXECUTION_FAILED",
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedLanguage(language) => {
                write!(formatter, "Unsupported language: {language:?}")
            }
            Self::InvalidInput(message) => write!(formatter, "Invalid input: {message}"),
            Self::GraphLoad(message) => write!(formatter, "Cannot load WFST: {message}"),
            Self::GraphExecution(message) => write!(formatter, "Cannot execute WFST: {message}"),
        }
    }
}

impl std::error::Error for AppError {}

/// A reusable normalizer that owns its compiled language graph.
pub struct Normalizer {
    runtime: Runtime,
}

impl Normalizer {
    /// Load a bundled language graph. Currently only `ml` is supported.
    pub fn new(language: &str) -> Result<Self, AppError> {
        let bytes: &[u8] = match language {
            "ml" => include_bytes!("../wfst/ml/assets/numbers.fst"),
            _ => return Err(AppError::UnsupportedLanguage(language.to_owned())),
        };
        Ok(Self {
            runtime: Runtime::load(bytes)?,
        })
    }

    /// Normalize supported expressions while preserving all other text.
    ///
    /// NUL is rejected because label zero represents epsilon in the graph.
    /// U+E000, U+E001, and U+E002 are reserved for fraction output markers.
    ///
    /// ```
    /// use inverse_text_normalization_rs::Normalizer;
    /// let normalizer = Normalizer::new("ml")?;
    /// assert_eq!(normalizer.normalize("അഞ്ച് രൂപ")?, "5 രൂപ");
    /// assert_eq!(
    ///     normalizer.normalize("ആയിരത്തി ഒരു നൂറ്റി ഇരുപത്തിയഞ്ച്")?,
    ///     "1125"
    /// );
    /// # Ok::<(), inverse_text_normalization_rs::AppError>(())
    /// ```
    pub fn normalize(&self, input: &str) -> Result<String, AppError> {
        if input.contains('\0') {
            return Err(AppError::InvalidInput("NUL is not supported"));
        }
        if input.contains(['\u{e000}', '\u{e001}', '\u{e002}']) {
            return Err(AppError::InvalidInput("Fraction markers are reserved"));
        }
        reorder_fractions(&self.runtime.apply(input)?)
    }
}

fn reorder_fractions(mut remaining: &str) -> Result<String, AppError> {
    let mut output = String::with_capacity(remaining.len());
    while let Some(index) = remaining.find(['\u{e000}', '\u{e001}', '\u{e002}']) {
        output.push_str(&remaining[..index]);
        let (denominator, rest) = remaining[index..]
            .strip_prefix('\u{e000}')
            .and_then(|fraction| fraction.split_once('\u{e001}'))
            .ok_or_else(|| AppError::GraphExecution("Malformed fraction markers".to_owned()))?;
        let (numerator, rest) = rest
            .split_once('\u{e002}')
            .ok_or_else(|| AppError::GraphExecution("Missing fraction end marker".to_owned()))?;
        if [denominator, numerator]
            .iter()
            .any(|operand| operand.is_empty() || !operand.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return Err(AppError::GraphExecution(
                "Fraction operands must be nonempty ASCII digits".to_owned(),
            ));
        }
        output.push_str(numerator);
        output.push('/');
        output.push_str(denominator);
        remaining = rest;
    }
    output.push_str(remaining);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{AppError, Normalizer, reorder_fractions};

    #[test]
    fn reorders_single_and_multiple_fractions() {
        for (raw, expected) in [
            ("\u{e000}5\u{e001}7\u{e002}", "7/5"),
            ("\u{e000}100\u{e001}999999999\u{e002}", "999999999/100"),
            ("\u{e000}7\u{e001}0\u{e002}", "0/7"),
            ("\u{e000}4\u{e001}2\u{e002}", "2/4"),
            (
                "\u{e000}5\u{e001}7\u{e002}\u{e000}2\u{e001}1\u{e002}",
                "7/51/2",
            ),
            (
                "ലോകം [\u{e000}5\u{e001}7\u{e002}],\n\u{e003}\u{e000}2\u{e001}1\u{e002}!",
                "ലോകം [7/5],\n\u{e003}1/2!",
            ),
            ("1\u{e000}5\u{e001}7\u{e002}2", "17/52"),
            ("\u{e000}005\u{e001}007\u{e002}", "007/005"),
        ] {
            assert_eq!(reorder_fractions(raw).unwrap(), expected, "{raw:?}");
        }
    }

    #[test]
    fn fraction_postprocessing_preserves_marker_free_unicode() {
        for input in [
            "",
            "hello, ലോകം! 7/5\n\t",
            "\0\u{d7ff}\u{e003}\u{f8ff}\u{f0000}\u{10fffd}",
        ] {
            assert_eq!(reorder_fractions(input).unwrap(), input);
        }
    }

    #[test]
    fn rejects_malformed_fraction_markers() {
        for raw in [
            "\u{e000}",
            "\u{e001}",
            "\u{e002}",
            "\u{e000}5",
            "\u{e000}5\u{e001}7",
            "\u{e000}5\u{e002}",
            "\u{e000}\u{e001}7\u{e002}",
            "\u{e000}5\u{e001}\u{e002}",
            "\u{e000}\u{e001}\u{e002}",
            "\u{e001}7\u{e000}5\u{e002}",
            "\u{e000}5\u{e002}7\u{e001}",
            "\u{e000}\u{e000}5\u{e001}7\u{e002}",
            "\u{e000}5\u{e001}\u{e000}2\u{e001}7\u{e002}\u{e002}",
            "\u{e000}5\u{e001}\u{e001}7\u{e002}",
            "\u{e000}5\u{e001}7\u{e002}\u{e000}",
            "\u{e000}5\u{e001}7\u{e002}\u{e001}",
            "\u{e000}5\u{e001}7\u{e002}\u{e002}",
            "\u{e002}\u{e000}5\u{e001}7\u{e002}",
            "\u{e000}a\u{e001}7\u{e002}",
            "\u{e000}5\u{e001}a\u{e002}",
            "\u{e000} 5\u{e001}7\u{e002}",
            "\u{e000}5\u{e001}7 \u{e002}",
            "\u{e000}-5\u{e001}7\u{e002}",
            "\u{e000}5\u{e001}+7\u{e002}",
            "\u{e000}5\u{e001}7/2\u{e002}",
            "\u{e000}5.0\u{e001}7\u{e002}",
            "\u{e000}൫\u{e001}7\u{e002}",
            "\u{e000}5\u{e001}٧\u{e002}",
            "\u{e000}5\u{e001}\u{e003}\u{e002}",
        ] {
            let error = reorder_fractions(raw).unwrap_err();
            assert!(matches!(error, AppError::GraphExecution(_)), "{raw:?}");
            assert_eq!(error.code(), "GRAPH_EXECUTION_FAILED");
        }
    }

    #[test]
    fn normalize_rejects_reserved_markers_and_nul() {
        let normalizer = Normalizer::new("ml").unwrap();
        for input in [
            "\u{e000}",
            "text\u{e001}text",
            "\u{e002}",
            "\u{e000}5\u{e001}7\u{e002}",
            "text\0text",
        ] {
            let error = normalizer.normalize(input).unwrap_err();
            assert!(matches!(error, AppError::InvalidInput(_)), "{input:?}");
            assert_eq!(error.code(), "INVALID_INPUT");
        }
    }

    #[test]
    fn normalize_preserves_unicode_and_unrelated_private_use() {
        let normalizer = Normalizer::new("ml").unwrap();
        for input in ["", "hello, ലോകം!", "\u{e003}\u{f8ff}\u{f0000}\u{10fffd}"] {
            assert_eq!(normalizer.normalize(input).unwrap(), input);
        }
    }
}
