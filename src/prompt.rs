//! Prompt system for interactive user input (feature "prompt-demand")
//!
//! Provides abstractions for interactive prompts using the `demand` crate.
//! Browser environments will return an error if prompts are attempted.
//! Note: WASM running in Node.js, Wasmer, or other runtimes may support prompts.

use std::error::Error as StdError;
use thiserror::Error;

/// Strategy for handling prompt cancellation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptCancelStrategy {
    /// Return an error on cancellation
    Reject,
    /// Return a default value on cancellation
    Default,
    /// Return Undefined outcome
    Undefined,
    /// Return Null value
    Null,
    /// Return a Symbol indicating cancellation
    Symbol,
}

/// Outcome of a prompt operation
#[derive(Debug, Clone, PartialEq)]
pub enum PromptOutcome<T> {
    /// Successfully obtained a value
    Value(T),
    /// Prompt was cancelled and strategy chose Undefined
    Undefined,
    /// Prompt was cancelled and strategy chose Null
    NullValue,
    /// Prompt was cancelled and strategy chose Symbol
    SymbolCancel,
    /// Prompt was cancelled and strategy was Reject
    Cancelled,
}

impl<T> PromptOutcome<T> {
    /// Unwrap the value or panic
    pub fn unwrap(self) -> T {
        match self {
            PromptOutcome::Value(v) => v,
            _ => panic!("called `PromptOutcome::unwrap()` on a non-Value variant"),
        }
    }

    /// Get the value or a default
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            PromptOutcome::Value(v) => v,
            _ => default,
        }
    }

    /// Check if this is a Value
    pub fn is_value(&self) -> bool {
        matches!(self, PromptOutcome::Value(_))
    }
}

/// Error type for prompt operations
#[derive(Debug, Error)]
pub enum PromptError {
    /// Prompt was cancelled by user
    #[error("prompt cancelled by user")]
    Cancelled,
    /// Prompt not supported (e.g., in browsers)
    #[error("prompts not supported in this environment (browser)")]
    NotSupported,
    /// Internal error from demand crate
    #[cfg(feature = "prompt-demand")]
    #[error("prompt error: {0}")]
    DemandError(String),
    /// Generic error
    #[error("prompt error: {0}")]
    Other(Box<dyn StdError + Send + Sync>),
}

/// Trait for prompt providers
pub trait PromptProvider: Send + Sync {
    /// Prompt for a text input
    fn text(
        &self,
        prompt: &str,
        default: Option<&str>,
    ) -> Result<PromptOutcome<String>, PromptError>;

    /// Prompt for a yes/no confirmation
    fn confirm(
        &self,
        prompt: &str,
        default: Option<bool>,
    ) -> Result<PromptOutcome<bool>, PromptError>;

    /// Prompt for a single selection from a list
    fn select(&self, prompt: &str, options: &[&str]) -> Result<PromptOutcome<usize>, PromptError>;

    /// Prompt for multiple selections from a list
    fn multiselect(
        &self,
        prompt: &str,
        options: &[&str],
    ) -> Result<PromptOutcome<Vec<usize>>, PromptError>;
}

/// Check if we're running in a browser environment
#[cfg(target_arch = "wasm32")]
fn is_browser() -> bool {
    #[cfg(feature = "wasm")]
    {
        // Use wasm_bindgen to check for browser-specific globals
        use wasm_bindgen::JsValue;

        // Try to get the global 'window' object
        let global = js_sys::global();
        let window = js_sys::Reflect::get(&global, &JsValue::from_str("window"));

        // If window exists and is not undefined, we're likely in a browser
        window.is_ok() && !window.unwrap().is_undefined()
    }
    #[cfg(not(feature = "wasm"))]
    {
        // If wasm feature isn't enabled but we're on wasm32, conservatively assume browser
        true
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_browser() -> bool {
    false
}

/// Default prompt provider using the demand crate
#[cfg(feature = "prompt-demand")]
pub struct DefaultDemandPrompt {
    cancel_strategy: PromptCancelStrategy,
}

#[cfg(feature = "prompt-demand")]
impl DefaultDemandPrompt {
    /// Create a new prompt provider with the given cancellation strategy
    pub fn new(cancel_strategy: PromptCancelStrategy) -> Self {
        Self { cancel_strategy }
    }

    /// Create a new prompt provider with default cancellation strategy (Reject)
    pub fn new_default() -> Self {
        Self::new(PromptCancelStrategy::Reject)
    }

    /// Map demand interruption to outcome based on strategy
    fn map_cancellation<T>(&self) -> PromptOutcome<T> {
        match self.cancel_strategy {
            PromptCancelStrategy::Reject => PromptOutcome::Cancelled,
            PromptCancelStrategy::Default => PromptOutcome::Cancelled, // Caller should handle default
            PromptCancelStrategy::Undefined => PromptOutcome::Undefined,
            PromptCancelStrategy::Null => PromptOutcome::NullValue,
            PromptCancelStrategy::Symbol => PromptOutcome::SymbolCancel,
        }
    }
}

#[cfg(feature = "prompt-demand")]
impl PromptProvider for DefaultDemandPrompt {
    fn text(
        &self,
        prompt: &str,
        default: Option<&str>,
    ) -> Result<PromptOutcome<String>, PromptError> {
        // Check if we're in a browser at runtime
        if is_browser() {
            return Err(PromptError::NotSupported);
        }

        let mut input = demand::Input::new(prompt);
        if let Some(def) = default {
            input = input.placeholder(def);
        }

        match input.run() {
            Ok(value) => Ok(PromptOutcome::Value(value)),
            Err(_e) => {
                // Assume interruption/cancellation
                if matches!(self.cancel_strategy, PromptCancelStrategy::Default) {
                    if let Some(def) = default {
                        Ok(PromptOutcome::Value(def.to_string()))
                    } else {
                        Ok(self.map_cancellation())
                    }
                } else {
                    Ok(self.map_cancellation())
                }
            }
        }
    }

    fn confirm(
        &self,
        prompt: &str,
        default: Option<bool>,
    ) -> Result<PromptOutcome<bool>, PromptError> {
        // Check if we're in a browser at runtime
        if is_browser() {
            return Err(PromptError::NotSupported);
        }

        let confirm = demand::Confirm::new(prompt);
        // Note: demand::Confirm doesn't expose a method to set default in v1.7

        match confirm.run() {
            Ok(value) => Ok(PromptOutcome::Value(value)),
            Err(_e) => {
                // Assume interruption/cancellation
                if matches!(self.cancel_strategy, PromptCancelStrategy::Default) {
                    if let Some(def) = default {
                        Ok(PromptOutcome::Value(def))
                    } else {
                        Ok(self.map_cancellation())
                    }
                } else {
                    Ok(self.map_cancellation())
                }
            }
        }
    }

    fn select(&self, prompt: &str, options: &[&str]) -> Result<PromptOutcome<usize>, PromptError> {
        // Check if we're in a browser at runtime
        if is_browser() {
            return Err(PromptError::NotSupported);
        }

        // For now, implement a simple text-based selection
        // This is a simplified implementation until we can properly use demand::Select
        let mut input_text = format!("{}\nOptions:\n", prompt);
        for (i, opt) in options.iter().enumerate() {
            input_text.push_str(&format!("{}. {}\n", i + 1, opt));
        }
        input_text.push_str("Enter number: ");

        let input = demand::Input::new(&input_text);
        match input.run() {
            Ok(value) => {
                if let Ok(num) = value.trim().parse::<usize>() {
                    if num > 0 && num <= options.len() {
                        Ok(PromptOutcome::Value(num - 1))
                    } else {
                        Ok(self.map_cancellation())
                    }
                } else {
                    Ok(self.map_cancellation())
                }
            }
            Err(_e) => Ok(self.map_cancellation()),
        }
    }

    fn multiselect(
        &self,
        prompt: &str,
        options: &[&str],
    ) -> Result<PromptOutcome<Vec<usize>>, PromptError> {
        // Check if we're in a browser at runtime
        if is_browser() {
            return Err(PromptError::NotSupported);
        }

        // For now, implement a simple text-based multi-selection
        let mut input_text = format!("{}\nOptions:\n", prompt);
        for (i, opt) in options.iter().enumerate() {
            input_text.push_str(&format!("{}. {}\n", i + 1, opt));
        }
        input_text.push_str("Enter numbers (comma-separated): ");

        let input = demand::Input::new(&input_text);
        match input.run() {
            Ok(value) => {
                let indices: Vec<usize> = value
                    .split(',')
                    .filter_map(|s| s.trim().parse::<usize>().ok())
                    .filter(|&n| n > 0 && n <= options.len())
                    .map(|n| n - 1)
                    .collect();
                Ok(PromptOutcome::Value(indices))
            }
            Err(_e) => Ok(self.map_cancellation()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancel_strategy_variants() {
        assert_eq!(PromptCancelStrategy::Reject, PromptCancelStrategy::Reject);
        assert_ne!(PromptCancelStrategy::Reject, PromptCancelStrategy::Default);
    }

    #[test]
    fn test_prompt_outcome_value() {
        let outcome = PromptOutcome::Value(42);
        assert!(outcome.is_value());
        assert_eq!(outcome.unwrap(), 42);
    }

    #[test]
    fn test_prompt_outcome_unwrap_or() {
        let outcome: PromptOutcome<i32> = PromptOutcome::Cancelled;
        assert_eq!(outcome.unwrap_or(99), 99);

        let outcome = PromptOutcome::Value(42);
        assert_eq!(outcome.unwrap_or(99), 42);
    }

    #[test]
    #[should_panic(expected = "called `PromptOutcome::unwrap()` on a non-Value variant")]
    fn test_prompt_outcome_unwrap_panics() {
        let outcome: PromptOutcome<i32> = PromptOutcome::Cancelled;
        outcome.unwrap();
    }

    #[cfg(feature = "prompt-demand")]
    #[test]
    fn test_default_demand_prompt_creation() {
        let prompt = DefaultDemandPrompt::new_default();
        // Just verify it constructs without error
        assert_eq!(prompt.cancel_strategy, PromptCancelStrategy::Reject);
    }

    #[test]
    fn test_is_browser_on_native() {
        // On native platforms, is_browser() should always return false
        #[cfg(not(target_arch = "wasm32"))]
        assert!(!is_browser());
    }
}
