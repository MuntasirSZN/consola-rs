// ─── Log object detection ─────────────────────────────────────────────────────
// Minimal stubs retained for API compatibility with JS codebase.

use crate::types::LogObjectInput;

/// Returns true — in Rust all struct dispatch is compile-time, no duck-typing needed.
pub fn is_plain_object<T>(_: &T) -> bool {
    true
}

/// In the JS version this checked whether a single argument is a log-object
/// (plain object with message/args, no stack). In Rust the type system handles this.
pub fn is_log_obj_args(_args: &[String]) -> bool {
    false
}

/// Check if a `LogObjectInput` should be treated as a structured log object.
pub fn is_log_object_input(input: &LogObjectInput) -> bool {
    input.message.is_some() || !input.args.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LogObjectInput;

    #[test]
    fn test_is_plain_object() {
        // Always returns true for any type
        assert!(is_plain_object(&42));
        assert!(is_plain_object(&"hello"));
        assert!(is_plain_object(&vec![1, 2, 3]));
    }

    #[test]
    fn test_is_log_obj_args() {
        // Always returns false in Rust — type system handles this
        assert!(!is_log_obj_args(&[]));
        assert!(!is_log_obj_args(&["a".to_string()]));
        assert!(!is_log_obj_args(&["a".to_string(), "b".to_string()]));
    }

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
