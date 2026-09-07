//! Error types for template directive execution.
//!
//! This module defines the errors that can occur during template rendering,
//! particularly when resolving arguments and executing directives.

use std::error::Error;
use std::fmt;

/// Errors that can occur during directive execution.
///
/// These errors represent runtime failures when rendering a compiled template,
/// such as missing variables, type mismatches, or parse failures. All errors
/// include detailed context to help diagnose issues.
///
/// # Examples
///
/// ```rust
/// use figura::{Template, DefaultParser, Context, Value, DirectiveError};
/// use std::collections::HashMap;
///
/// let tmpl = Template::<'{', '}'>::compile::<DefaultParser>("{missing}").unwrap();
/// let ctx = HashMap::new();
///
/// match tmpl.format(&ctx) {
///     Err(DirectiveError::NotFound { name, .. }) => {
///         println!("Variable '{}' not found", name);
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug)]
pub enum DirectiveError {
    /// A variable was not found in the template context.
    ///
    /// This error occurs when a template references a variable that doesn't
    /// exist in the context hashmap. The error includes both the variable name
    /// and the type it was expected to be used as.
    ///
    /// # Fields
    ///
    /// * `name` - The name of the missing variable
    /// * `type_name` - The type the variable was expected to be (e.g., "string", "i64")
    ///
    /// # Examples
    ///
    /// ```text
    /// Template: "{username}"
    /// Context: (empty)
    /// Error: Variable 'username' was not found in the context while being used as 'string'
    /// ```
    NotFound {
        name: String,
        type_name: &'static str,
    },
    /// A variable was found but has an incompatible type.
    ///
    /// This error occurs when a variable exists in the context but cannot be
    /// converted to the required type for the operation. For example, trying
    /// to use a string value where an integer is required.
    ///
    /// # Fields
    ///
    /// * `name` - The name of the variable with the wrong type
    /// * `expected` - The type that was expected (e.g., "i64")
    /// * `found` - The actual type of the variable (e.g., "string")
    ///
    /// # Examples
    ///
    /// ```text
    /// Template: "{value:count}"  (requires count to be an integer)
    /// Context: count = "not a number"
    /// Error: Variable 'count' has type 'string' but was expected to have type 'i64'
    /// ```
    TypeError {
        name: String,
        expected: &'static str,
        found: String,
    },
    /// A literal value could not be parsed as the required type.
    ///
    /// This error occurs when a literal value embedded in the template cannot
    /// be parsed into the type required by the directive. For example, trying
    /// to parse "abc" as an integer.
    ///
    /// # Fields
    ///
    /// * `value` - The literal value that failed to parse
    /// * `type_name` - The type it was being parsed as
    /// * `message` - The detailed parse error message
    ///
    /// # Examples
    ///
    /// ```text
    /// Template: "{'hello':3}"  (tries to parse "hello" as i64 for repeat count)
    /// Error: Failed to parse 'hello' as a literal of type 'i64': invalid digit found in string
    /// ```
    ParseError {
        value: String,
        type_name: &'static str,
        message: String,
    },
}

impl fmt::Display for DirectiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { name, type_name } => write!(
                f,
                "Variable '{name}' was not found in the context while being used as '{type_name}'"
            ),
            Self::TypeError {
                name,
                expected,
                found,
            } => write!(
                f,
                "Variable '{name}' has type '{found}' but was expected to have type '{expected}'"
            ),
            Self::ParseError {
                value,
                type_name,
                message,
            } => write!(
                f,
                "Failed to parse '{value}' as a literal of type '{type_name}': {message}"
            ),
        }
    }
}

impl Error for DirectiveError {}

#[derive(Debug)]
pub enum TemplateError {
    MissingDelimiter(char),
    DirectiveParsing(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDelimiter(delim) => write!(f, "Unclosed delimiter '{delim}'"),
            Self::DirectiveParsing(msg) => write!(f, "Failed to parse directive: {msg}"),
        }
    }
}

impl Error for TemplateError {}
