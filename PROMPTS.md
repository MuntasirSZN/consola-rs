# Interactive Prompts Guide

This guide explains how to use interactive prompts in consola-rs powered by the `demand` crate.

## Overview

consola-rs provides interactive prompt capabilities through the optional `prompt-demand` feature, powered by the excellent [demand](https://docs.rs/demand) crate. Prompts allow you to collect user input in a rich, interactive way with validation and cancellation support.

⚠️ **WASM Limitation**: Interactive prompts are **not available in WASM targets**. Calling prompt methods in WASM browser environments will return an error.

## Enabling Prompts

Add the feature to your `Cargo.toml`:

```toml
[dependencies]
consola = { version = "0.0.0-alpha.0", features = ["prompt-demand"] }
```

Or for native-only builds:

```toml
[dependencies]
consola = { version = "0.0.0-alpha.0", default-features = false, features = ["color", "fancy", "prompt-demand"] }
```

## Basic Usage

### Text Input

```rust
use consola::*;

fn main() -> anyhow::Result<()> {
    let mut logger = BasicLogger::default();
    
    // Simple text input
    let name = logger.prompt_text("What is your name?")?;
    logger.log("info", None, [format!("Hello, {}!", name)]);
    
    Ok(())
}
```

### Confirmation

```rust
let confirmed = logger.prompt_confirm("Do you want to continue?")?;
if confirmed {
    logger.log("info", None, ["Continuing..."]);
} else {
    logger.log("warn", None, ["Operation cancelled"]);
}
```

### Selection

```rust
let options = vec!["Option 1", "Option 2", "Option 3"];
let choice = logger.prompt_select("Choose an option:", &options)?;
logger.log("info", None, [format!("You selected: {}", choice)]);
```

### Multi-Selection

```rust
let options = vec!["Feature A", "Feature B", "Feature C"];
let selections = logger.prompt_multiselect("Select features:", &options)?;
logger.log("info", None, [format!("Selected {} features", selections.len())]);
```

## Prompt Cancellation

Users can cancel prompts (typically with Ctrl+C). You control how cancellation is handled using `PromptCancelStrategy`:

```rust
use consola::{PromptCancelStrategy, PromptOutcome};

// Different cancellation strategies
let strategy = PromptCancelStrategy::Reject;        // Return error
let strategy = PromptCancelStrategy::Default(..);   // Use default value
let strategy = PromptCancelStrategy::Undefined;     // Return Undefined
let strategy = PromptCancelStrategy::Null;          // Return NullValue
let strategy = PromptCancelStrategy::Symbol;        // Return SymbolCancel
```

### Handling Cancellation

```rust
match logger.prompt_text_with_cancel("Enter name:", PromptCancelStrategy::Reject) {
    Ok(PromptOutcome::Value(name)) => {
        logger.log("info", None, [format!("Name: {}", name)]);
    }
    Ok(PromptOutcome::Cancelled) => {
        logger.log("warn", None, ["User cancelled"]);
    }
    Err(e) => {
        logger.log("error", None, [format!("Error: {}", e)]);
    }
    _ => {}
}
```

### Using Default Values

```rust
let name = logger.prompt_text_with_cancel(
    "Enter name (or press Ctrl+C for default):",
    PromptCancelStrategy::Default("Guest".to_string())
)?;

// If user cancels, `name` will be "Guest"
```

## Cancellation Strategy Table

| Strategy | Behavior | Use Case |
|----------|----------|----------|
| `Reject` | Returns `Err` | Critical input, must be provided |
| `Default(T)` | Returns provided value | Optional input with fallback |
| `Undefined` | Returns `PromptOutcome::Undefined` | Distinguish between empty and cancelled |
| `Null` | Returns `PromptOutcome::NullValue` | Null-aware applications |
| `Symbol` | Returns `PromptOutcome::SymbolCancel` | Symbolic cancellation marker |

## Advanced Prompt Customization

### Text Input with Validation

```rust
// Using demand directly for advanced features
#[cfg(feature = "prompt-demand")]
{
    use demand::{Input, Confirm};
    
    let input = Input::new("Enter email:")
        .placeholder("user@example.com")
        .validate(|val| {
            if val.contains('@') {
                Ok(())
            } else {
                Err("Invalid email address".to_string())
            }
        });
    
    match input.run() {
        Ok(email) => logger.log("info", None, [format!("Email: {}", email)]),
        Err(e) => logger.log("error", None, [format!("Input error: {}", e)]),
    }
}
```

### Multi-Step Prompts

```rust
fn collect_user_info(logger: &mut BasicLogger) -> anyhow::Result<UserInfo> {
    logger.log("info", None, ["Please provide your information:"]);
    
    let name = logger.prompt_text("Name:")?;
    let age: u32 = logger.prompt_text("Age:")?.parse()?;
    
    let roles = vec!["Admin", "User", "Guest"];
    let role = logger.prompt_select("Role:", &roles)?;
    
    let confirmed = logger.prompt_confirm("Is this information correct?")?;
    
    if !confirmed {
        logger.log("warn", None, ["Please start over"]);
        return collect_user_info(logger);
    }
    
    Ok(UserInfo { name, age, role })
}
```

## WASM Behavior

When compiling for WASM (with the `wasm` feature), prompt methods will:

1. Return `Err(anyhow::Error)` with a descriptive message
2. Return an error if called in browser environments
3. Not block or hang the application

### WASM Example

```rust
#[cfg(target_arch = "wasm32")]
{
    // This will return an error in WASM browser environments
    match logger.prompt_text("This won't work in WASM") {
        Ok(_) => unreachable!("Prompts don't work in WASM browser environments"),
        Err(e) => {
            // Expected: "Interactive prompts are not available in WASM targets"
            logger.log("error", None, [format!("Prompt error: {}", e)]);
        }
    }
}
```

### Conditional Compilation

For cross-platform code, use conditional compilation:

```rust
#[cfg(not(target_arch = "wasm32"))]
{
    // Native: use prompts
    let input = logger.prompt_text("Enter value:")?;
    process(input);
}

#[cfg(target_arch = "wasm32")]
{
    // WASM: use alternative approach
    let input = get_from_form_or_api().await?;
    process(input);
}
```

## The PromptProvider Trait

For testing or custom prompt implementations, implement the `PromptProvider` trait:

```rust
#[cfg(feature = "prompt-demand")]
pub trait PromptProvider {
    fn text(&self, prompt: &str) -> Result<String>;
    fn confirm(&self, prompt: &str) -> Result<bool>;
    fn select(&self, prompt: &str, options: &[&str]) -> Result<String>;
    fn multiselect(&self, prompt: &str, options: &[&str]) -> Result<Vec<String>>;
    
    fn text_with_cancel(&self, prompt: &str, strategy: PromptCancelStrategy<String>) 
        -> Result<PromptOutcome<String>>;
    // ... other methods with cancellation support
}
```

### Mock Provider for Testing

```rust
#[cfg(test)]
struct MockPromptProvider {
    responses: Vec<String>,
    index: std::cell::RefCell<usize>,
}

impl PromptProvider for MockPromptProvider {
    fn text(&self, _prompt: &str) -> Result<String> {
        let mut idx = self.index.borrow_mut();
        let response = self.responses[*idx].clone();
        *idx += 1;
        Ok(response)
    }
    
    // Implement other methods...
}

#[test]
fn test_with_mock_prompts() {
    let mock = MockPromptProvider {
        responses: vec!["Alice".to_string(), "30".to_string()],
        index: RefCell::new(0),
    };
    
    // Use mock in tests
    // logger.with_prompt_provider(mock);
}
```

## Integration with demand Crate

The `demand` crate provides rich prompt features. You can use it directly for advanced scenarios:

```rust
#[cfg(feature = "prompt-demand")]
use demand::{Input, Select, MultiSelect, Confirm, DemandOption};

// Advanced input with placeholders and history
let input = Input::new("Enter command:")
    .placeholder("ls -la")
    .suggestions(&["ls", "cd", "pwd", "cat"])
    .run()?;

// Custom styling
let select = Select::new("Choose theme:")
    .description("Select your preferred color theme")
    .item(DemandOption::new("dark", "Dark Mode"))
    .item(DemandOption::new("light", "Light Mode"))
    .item(DemandOption::new("auto", "Auto (System)"))
    .run()?;
```

## Error Handling

Prompts can fail for various reasons:

```rust
match logger.prompt_text("Enter value:") {
    Ok(value) => {
        // Success
    }
    Err(e) => {
        // Handle errors:
        // - User interrupted (Ctrl+C with Reject strategy)
        // - Terminal not available
        // - WASM browser environment
        // - IO errors
        logger.log("error", None, [format!("Prompt failed: {}", e)]);
    }
}
```

## Best Practices

1. **Provide clear prompts**: Make questions unambiguous
   ```rust
   // Good
   let name = logger.prompt_text("Enter your full name:")?;
   
   // Better
   let name = logger.prompt_text("Enter your full name (e.g., John Doe):")?;
   ```

2. **Use appropriate cancellation strategies**:
   - Required input → `Reject`
   - Optional input → `Default(value)`
   - User choice → `Undefined`

3. **Validate input early**:
   ```rust
   let age_str = logger.prompt_text("Enter age (1-120):")?;
   let age: u32 = age_str.parse()
       .map_err(|_| anyhow::anyhow!("Invalid number"))?;
   
   if age == 0 || age > 120 {
       return Err(anyhow::anyhow!("Age must be between 1 and 120"));
   }
   ```

4. **Confirm destructive actions**:
   ```rust
   let confirmed = logger.prompt_confirm("Delete all files? This cannot be undone.")?;
   if !confirmed {
       logger.log("info", None, ["Operation cancelled"]);
       return Ok(());
   }
   ```

5. **Handle WASM gracefully**:
   ```rust
   #[cfg(feature = "prompt-demand")]
   #[cfg(not(target_arch = "wasm32"))]
   fn interactive_setup(logger: &mut BasicLogger) {
       // Safe to use prompts
   }
   
   #[cfg(target_arch = "wasm32")]
   fn interactive_setup(logger: &mut BasicLogger) {
       logger.log("warn", None, ["Interactive setup not available in browser"]);
       // Provide alternative mechanism
   }
   ```

## Platform Support

| Platform | Support | Notes |
|----------|---------|-------|
| Linux | ✅ Full | All features supported |
| macOS | ✅ Full | All features supported |
| Windows | ✅ Full | Requires Windows 10+ for best experience |
| WASM (browser) | ❌ Not supported | Returns error, use form inputs instead |

## Troubleshooting

### "Prompts don't work in my terminal"

Some terminals may not support interactive features. Ensure:
- Terminal is a TTY (not piped/redirected)
- Terminal supports ANSI escape codes
- stdin is not redirected

### "Prompts hang or freeze"

Check for:
- Signal handlers interfering with Ctrl+C
- Terminal in raw mode from previous operations
- Conflicting readline configurations

### "Prompts return errors in WASM"

This is expected behavior in browser environments. Prompts are not supported in browsers. For WASM targets, avoid enabling `prompt-demand`:

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
consola = { version = "0.0.0-alpha.0", features = ["prompt-demand"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
consola = { version = "0.0.0-alpha.0", features = ["wasm"] }
```

## Examples

See `examples/prompts.rs` in the repository for complete working examples.

## See Also

- [demand crate documentation](https://docs.rs/demand) - Underlying prompt library
- [REPORTERS.md](REPORTERS.md) - Custom reporters
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [WASM documentation](README.md#wasm-support) - WASM limitations and workarounds
