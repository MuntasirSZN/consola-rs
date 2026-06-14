//! Utility functions for detecting log object inputs.

use crate::types::LogObjectInput;

/// Check if a `LogObjectInput` should be treated as a structured log object.
pub fn is_log_object_input(input: &LogObjectInput) -> bool {
    input.message.is_some() || !input.args.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LogObjectInput;

    #[test]
    fn test_is_log_object_input_empty() {
        let input = LogObjectInput::new();
        // message is None and args is empty → false
        assert!(!is_log_object_input(&input));
    }

    #[test]
    fn test_is_log_object_input_with_message() {
        let input = LogObjectInput::new().message("hello");
        assert!(is_log_object_input(&input));
    }

    #[test]
    fn test_is_log_object_input_with_args() {
        let input = LogObjectInput {
            args: vec!["arg".into()],
            ..Default::default()
        };
        assert!(is_log_object_input(&input));
    }

    #[test]
    fn test_is_log_object_input_with_both() {
        let input = LogObjectInput {
            message: Some("msg".into()),
            args: vec!["arg".into()],
            ..Default::default()
        };
        assert!(is_log_object_input(&input));
    }
}
