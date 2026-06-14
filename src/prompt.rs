//! Interactive prompts for user input.
//!
//! Provides [`text()`], [`confirm()`], [`select()`], and [`multiselect()`] functions,
//! plus the sentinel constant [`K_CANCEL`] returned when the user aborts.
//!
//! Backend selection (priority):
//! - `prompt` → demand (default)
//! - `prompt-inquire` → inquire
//! - `prompt-dialoguer` → dialoguer

/// Sentinel value returned when a user cancels a prompt.
pub const K_CANCEL: &str = "Symbol(cancel)";

pub use crate::types::{
    ConfirmPromptOptions, MultiSelectOptions, PromptCommonOptions, PromptOptions, SelectOption,
    SelectPromptOptions, TextPromptOptions,
};

/// Demand backend (highest priority).
#[cfg(feature = "prompt")]
mod backend {
    use super::*;
    use demand::{Confirm, Input, MultiSelect, Select};

    pub(super) fn text(message: &str, opts: &TextPromptOptions) -> Result<String, String> {
        let mut input = Input::new(message);
        if let Some(placeholder) = &opts.placeholder {
            input = input.placeholder(placeholder);
        }
        if let Some(default) = &opts.default {
            input = input.default_value(default);
        }
        input.run().map_err(map_err_demand)
    }

    pub(super) fn confirm(message: &str, opts: &ConfirmPromptOptions) -> Result<bool, String> {
        let mut dialog = Confirm::new(message);
        if let Some(initial) = opts.initial {
            dialog = dialog.selected(initial);
        }
        dialog.run().map_err(map_err_demand)
    }

    pub(super) fn select(message: &str, opts: &SelectPromptOptions) -> Result<String, String> {
        let items: Vec<demand::DemandOption<String>> = opts
            .options
            .iter()
            .map(|opt| {
                let mut d = demand::DemandOption::with_label(&opt.label, opt.value.clone());
                if let Some(hint) = &opt.hint {
                    d = d.description(hint);
                }
                d
            })
            .collect();
        Select::new(message)
            .options(items)
            .run()
            .map_err(map_err_demand)
    }

    pub(super) fn multiselect(
        message: &str,
        opts: &MultiSelectOptions,
    ) -> Result<Vec<String>, String> {
        let mut ms = MultiSelect::new(message);
        if opts.required.unwrap_or(false) {
            ms = ms.min(1);
        }
        let items: Vec<demand::DemandOption<String>> = opts
            .options
            .iter()
            .map(|opt| {
                let mut d = demand::DemandOption::with_label(&opt.label, opt.value.clone());
                if let Some(hint) = &opt.hint {
                    d = d.description(hint);
                }
                d
            })
            .collect();
        ms.options(items).run().map_err(map_err_demand)
    }

    fn map_err_demand(e: std::io::Error) -> String {
        if e.kind() == std::io::ErrorKind::Interrupted {
            K_CANCEL.to_string()
        } else {
            e.to_string()
        }
    }
}

/// Inquire backend (middle priority).
#[cfg(all(feature = "prompt-inquire", not(feature = "prompt")))]
mod backend {
    use super::*;

    pub(super) fn text(message: &str, opts: &TextPromptOptions) -> Result<String, String> {
        let mut input = inquire::Text::new(message);
        if let Some(placeholder) = &opts.placeholder {
            input = input.with_placeholder(placeholder);
        }
        if let Some(default) = &opts.default {
            input = input.with_default(default);
        }
        input.prompt().map_err(|e| e.to_string())
    }

    pub(super) fn confirm(message: &str, opts: &ConfirmPromptOptions) -> Result<bool, String> {
        let mut dialog = inquire::Confirm::new(message);
        if let Some(initial) = opts.initial {
            dialog = dialog.with_default(initial);
        }
        dialog.prompt().map_err(|e| e.to_string())
    }

    pub(super) fn select(message: &str, opts: &SelectPromptOptions) -> Result<String, String> {
        let labels: Vec<String> = opts.options.iter().map(|o| o.label.clone()).collect();
        let sel = inquire::Select::new(message, labels.clone())
            .prompt()
            .map_err(|e| e.to_string())?;
        let i = labels.iter().position(|s| *s == sel).unwrap();
        Ok(opts.options[i].value.clone())
    }

    pub(super) fn multiselect(
        message: &str,
        opts: &MultiSelectOptions,
    ) -> Result<Vec<String>, String> {
        let labels: Vec<String> = opts.options.iter().map(|o| o.label.clone()).collect();
        let selected = inquire::MultiSelect::new(message, labels.clone())
            .prompt()
            .map_err(|e| e.to_string())?;
        Ok(selected
            .iter()
            .map(|sel| {
                let i = labels.iter().position(|s| *s == *sel).unwrap();
                opts.options[i].value.clone()
            })
            .collect())
    }
}

/// Dialoguer backend (lowest priority).
#[cfg(all(
    feature = "prompt-dialoguer",
    not(any(feature = "prompt", feature = "prompt-inquire"))
))]
mod backend {
    use super::*;

    pub(super) fn text(message: &str, opts: &TextPromptOptions) -> Result<String, String> {
        let mut input = dialoguer::Input::<String>::new().with_prompt(message);
        if let Some(default) = &opts.default {
            input = input.with_initial_text(default);
        }
        input
            .allow_empty(true)
            .interact_text()
            .map_err(|e| e.to_string())
    }

    pub(super) fn confirm(message: &str, opts: &ConfirmPromptOptions) -> Result<bool, String> {
        let mut dialog = dialoguer::Confirm::new().with_prompt(message);
        if let Some(initial) = opts.initial {
            dialog = dialog.default(initial);
        }
        dialog.interact().map_err(|e| e.to_string())
    }

    pub(super) fn select(message: &str, opts: &SelectPromptOptions) -> Result<String, String> {
        let items: Vec<&str> = opts.options.iter().map(|o| o.label.as_str()).collect();
        let i = dialoguer::Select::new()
            .with_prompt(message)
            .items(&items)
            .interact()
            .map_err(|e| e.to_string())?;
        Ok(opts.options[i].value.clone())
    }

    pub(super) fn multiselect(
        message: &str,
        opts: &MultiSelectOptions,
    ) -> Result<Vec<String>, String> {
        let items: Vec<&str> = opts.options.iter().map(|o| o.label.as_str()).collect();
        let selected = dialoguer::MultiSelect::new()
            .with_prompt(message)
            .items(&items)
            .interact()
            .map_err(|e| e.to_string())?;
        Ok(selected
            .iter()
            .map(|&i| opts.options[i].value.clone())
            .collect())
    }
}

/// Stub backend when no prompt feature is enabled.
#[cfg(not(any(
    feature = "prompt",
    feature = "prompt-inquire",
    feature = "prompt-dialoguer"
)))]
mod backend {
    use super::*;

    pub(super) fn text(_message: &str, _opts: &TextPromptOptions) -> Result<String, String> {
        Err("interactive prompts require the `prompt`, `prompt-inquire`, or `prompt-dialoguer` Cargo feature".into())
    }

    pub(super) fn confirm(_message: &str, _opts: &ConfirmPromptOptions) -> Result<bool, String> {
        Err("interactive prompts require the `prompt`, `prompt-inquire`, or `prompt-dialoguer` Cargo feature".into())
    }

    pub(super) fn select(_message: &str, _opts: &SelectPromptOptions) -> Result<String, String> {
        Err("interactive prompts require the `prompt`, `prompt-inquire`, or `prompt-dialoguer` Cargo feature".into())
    }

    pub(super) fn multiselect(
        _message: &str,
        _opts: &MultiSelectOptions,
    ) -> Result<Vec<String>, String> {
        Err("interactive prompts require the `prompt`, `prompt-inquire`, or `prompt-dialoguer` Cargo feature".into())
    }
}

/// Prompt the user for free-form text input.
pub fn text(message: &str, opts: &TextPromptOptions) -> Result<String, String> {
    backend::text(message, opts)
}

/// Prompt the user for a yes/no confirmation.
pub fn confirm(message: &str, opts: &ConfirmPromptOptions) -> Result<bool, String> {
    backend::confirm(message, opts)
}

/// Prompt the user to select a single option from a list.
pub fn select(message: &str, opts: &SelectPromptOptions) -> Result<String, String> {
    backend::select(message, opts)
}

/// Prompt the user to select one or more options from a list.
pub fn multiselect(message: &str, opts: &MultiSelectOptions) -> Result<Vec<String>, String> {
    backend::multiselect(message, opts)
}
