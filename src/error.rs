//! Typed error definitions for the consola library.

use thiserror::Error;

/// Errors that can occur during consola operations.
#[derive(Error, Debug, PartialEq)]
pub enum ConsolaError {
    /// The requested reporter index is out of bounds.
    #[error("reporter index {index} is out of bounds (length: {len})")]
    ReporterIndexOutOfBounds {
        /// The index that was requested.
        index: usize,
        /// The number of reporters currently registered.
        len: usize,
    },

    /// An interactive prompt operation failed.
    #[error("prompt error: {0}")]
    Prompt(String),

    /// The user cancelled an interactive prompt.
    #[error("prompt cancelled")]
    PromptCancelled,

    /// No prompt backend is available (need feature `prompt`, `prompt-inquire`, or `prompt-dialoguer`).
    #[error(
        "interactive prompts require the `prompt`, `prompt-inquire`, or `prompt-dialoguer` Cargo feature"
    )]
    NoPromptBackend,

    /// A reporter failed to format a log entry.
    #[error("reporter error: {0}")]
    Reporter(String),

    /// A value lookup failed unexpectedly.
    #[error("lookup failed: {0}")]
    Lookup(String),
}

impl From<String> for ConsolaError {
    fn from(s: String) -> Self {
        ConsolaError::Prompt(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reporter_index_out_of_bounds_display() {
        let err = ConsolaError::ReporterIndexOutOfBounds { index: 5, len: 3 };
        assert_eq!(
            err.to_string(),
            "reporter index 5 is out of bounds (length: 3)"
        );
    }

    #[test]
    fn test_reporter_index_out_of_bounds_debug() {
        let err = ConsolaError::ReporterIndexOutOfBounds { index: 0, len: 0 };
        let debug = format!("{:?}", err);
        assert!(debug.contains("ReporterIndexOutOfBounds"));
        assert!(debug.contains("index: 0"));
        assert!(debug.contains("len: 0"));
    }

    #[test]
    fn test_prompt_display() {
        let err = ConsolaError::Prompt("cancelled by user".into());
        assert_eq!(err.to_string(), "prompt error: cancelled by user");
    }

    #[test]
    fn test_prompt_debug() {
        let err = ConsolaError::Prompt("io error".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Prompt"));
        assert!(debug.contains("io error"));
    }

    #[test]
    fn test_prompt_cancelled_display() {
        let err = ConsolaError::PromptCancelled;
        assert_eq!(err.to_string(), "prompt cancelled");
    }

    #[test]
    fn test_prompt_cancelled_debug() {
        let err = ConsolaError::PromptCancelled;
        let debug = format!("{:?}", err);
        assert!(debug.contains("PromptCancelled"));
    }

    #[test]
    fn test_no_prompt_backend_display() {
        let err = ConsolaError::NoPromptBackend;
        assert_eq!(
            err.to_string(),
            "interactive prompts require the `prompt`, `prompt-inquire`, or `prompt-dialoguer` Cargo feature"
        );
    }

    #[test]
    fn test_no_prompt_backend_debug() {
        let err = ConsolaError::NoPromptBackend;
        let debug = format!("{:?}", err);
        assert!(debug.contains("NoPromptBackend"));
    }

    #[test]
    fn test_reporter_error_display() {
        let err = ConsolaError::Reporter("serialization failed".into());
        assert_eq!(err.to_string(), "reporter error: serialization failed");
    }

    #[test]
    fn test_reporter_error_debug() {
        let err = ConsolaError::Reporter("oops".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Reporter"));
        assert!(debug.contains("oops"));
    }

    #[test]
    fn test_lookup_display() {
        let err = ConsolaError::Lookup("key not found".into());
        assert_eq!(err.to_string(), "lookup failed: key not found");
    }

    #[test]
    fn test_lookup_debug() {
        let err = ConsolaError::Lookup("missing".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Lookup"));
        assert!(debug.contains("missing"));
    }

    #[test]
    fn test_from_string_conversion() {
        let err: ConsolaError = "custom message".to_string().into();
        assert_eq!(err.to_string(), "prompt error: custom message");
    }

    #[test]
    fn test_result_usage() {
        // Verify the error type can be used as an Err variant
        let ok: Result<i32, ConsolaError> = Ok(42);
        assert_eq!(ok, Ok(42));

        let err: Result<i32, ConsolaError> = Err(ConsolaError::Prompt("fail".into()));
        assert!(err.is_err());
        assert!(matches!(err, Err(ConsolaError::Prompt(s)) if s == "fail"));
    }

    #[test]
    fn test_error_trait_is_satisfied() {
        // The Error trait is automatically derived by thiserror
        fn assert_error<E: std::error::Error>() {}
        assert_error::<ConsolaError>();
    }

    #[test]
    fn test_source_returns_none_for_unit_variants() {
        use std::error::Error;
        assert!(ConsolaError::PromptCancelled.source().is_none());
        assert!(ConsolaError::NoPromptBackend.source().is_none());
    }

    #[test]
    fn test_source_returns_none_for_basic_variants() {
        use std::error::Error;
        assert!(ConsolaError::Prompt("x".into()).source().is_none());
        assert!(ConsolaError::Reporter("x".into()).source().is_none());
        assert!(ConsolaError::Lookup("x".into()).source().is_none());
        assert!(
            ConsolaError::ReporterIndexOutOfBounds { index: 0, len: 0 }
                .source()
                .is_none()
        );
    }

    #[test]
    fn test_send_sync() {
        // ConsolaError must be Send + Sync for the Reporter trait
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ConsolaError>();
        assert_sync::<ConsolaError>();
    }

    #[test]
    fn test_variant_values_accessible() {
        let err = ConsolaError::ReporterIndexOutOfBounds { index: 3, len: 10 };
        if let ConsolaError::ReporterIndexOutOfBounds { index, len } = &err {
            assert_eq!(*index, 3);
            assert_eq!(*len, 10);
        } else {
            panic!("wrong variant");
        }

        let prompt = ConsolaError::Prompt("msg".into());
        if let ConsolaError::Prompt(s) = &prompt {
            assert_eq!(s, "msg");
        } else {
            panic!("wrong variant");
        }
    }
}
