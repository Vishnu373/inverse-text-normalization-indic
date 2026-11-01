//! WFST inverse text normalization for Malayalam whole numbers.

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
            "ml" => include_bytes!("../ml/assets/numbers.fst"),
            _ => return Err(AppError::UnsupportedLanguage(language.to_owned())),
        };
        Ok(Self {
            runtime: Runtime::load(bytes)?,
        })
    }

    /// Normalize supported expressions while preserving all other text.
    ///
    /// NUL is rejected because label zero represents epsilon in the graph.
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
        self.runtime.apply(input)
    }
}
