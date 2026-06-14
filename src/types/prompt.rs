//! Prompt option types for interactive user input dialogs.

/// A single selectable option in a prompt menu.
#[derive(Debug, Clone)]
pub struct SelectOption {
    /// Display label shown to the user.
    pub label: String,
    /// Value returned when this option is selected.
    pub value: String,
    /// Optional hint text displayed alongside the label.
    pub hint: Option<String>,
}

/// Common options shared across all prompt types.
#[derive(Debug, Clone)]
pub struct PromptCommonOptions {
    /// Optional cancellation token; set to abort the prompt.
    pub cancel: Option<String>,
}

/// Options for a text input prompt.
#[derive(Debug, Clone)]
pub struct TextPromptOptions {
    /// Shared prompt options.
    pub common: PromptCommonOptions,
    /// Optional type override for the input (e.g. "password").
    pub r#type: Option<String>,
    /// Default value returned if the user provides no input.
    pub default: Option<String>,
    /// Placeholder text displayed inside the input field.
    pub placeholder: Option<String>,
    /// Initial pre-filled value.
    pub initial: Option<String>,
}

/// Options for a yes/no confirmation prompt.
#[derive(Debug, Clone)]
pub struct ConfirmPromptOptions {
    /// Shared prompt options.
    pub common: PromptCommonOptions,
    /// Type identifier for the confirm prompt.
    pub r#type: String,
    /// Default boolean state.
    pub initial: Option<bool>,
}

/// Options for a single-select prompt.
#[derive(Debug, Clone)]
pub struct SelectPromptOptions {
    /// Shared prompt options.
    pub common: PromptCommonOptions,
    /// Type identifier for the select prompt.
    pub r#type: String,
    /// Value of the initially selected option.
    pub initial: Option<String>,
    /// Available options to choose from.
    pub options: Vec<SelectOption>,
}

/// Options for a multi-select prompt.
#[derive(Debug, Clone)]
pub struct MultiSelectOptions {
    /// Shared prompt options.
    pub common: PromptCommonOptions,
    /// Type identifier for the multi-select prompt.
    pub r#type: String,
    /// Values of the initially selected options.
    pub initial: Option<Vec<String>>,
    /// Available options to choose from.
    pub options: Vec<SelectOption>,
    /// Whether at least one selection is required.
    pub required: Option<bool>,
}

/// Union of all supported prompt option types.
#[derive(Debug, Clone)]
pub enum PromptOptions {
    /// Free-form text input.
    Text(TextPromptOptions),
    /// Yes/no confirmation.
    Confirm(ConfirmPromptOptions),
    /// Single selection from a list.
    Select(SelectPromptOptions),
    /// Multiple selection from a list.
    MultiSelect(MultiSelectOptions),
}
