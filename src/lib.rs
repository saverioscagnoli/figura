//! # Figura
//!
//! Figura is a template engine for Rust that supports variable substitution,
//! repeating patterns, and conditional logic with customizable delimiters.
//!
//! If you don't like that, you can extend the default parser by implementing the `Parser` trait.
//!
//! ## Features
//!
//! - **Variable substitution**: `{name}` - Replace with context values
//! - **Repeating patterns**: `{pattern:count}` - Repeat a pattern N times
//! - **Conditionals**: `{condition ? true_value : false_value}` - Ternary expressions
//! - **Comparisons**: Support for `==`, `!=`, `>`, `<`, `>=`, `<=`
//! - **Custom Logic**: You can implement custom logic using the `Logic` and `Parser` traits
//! - **Custom delimiters**: Use any characters as open/close delimiters
//! - **Zero-copy where possible**: Leverages `Cow` for efficiency
//!
//! ## Example
//!
//! ```rust
//! use figura::{Template, Context, Value};
//! use std::collections::HashMap;
//!
//! // Create a context with variables
//! let mut ctx = HashMap::new();
//! ctx.insert("name", Value::static_str("World"));
//! ctx.insert("count", Value::Int(3));
//!
//! // Compile a template (using default '{' and '}' delimiters)
//! let template = Template::<'{', '}'>::compile(
//!     "Hello {name}! {'*':count}"
//! ).unwrap();
//!
//! // Render the template
//! let result = template.format(&ctx).unwrap();
//! assert_eq!(result, "Hello World! ***");
//! ```

#![warn(clippy::use_self)]
#![allow(clippy::should_implement_trait)]

mod arg;
mod directive;
mod err;
mod lexer;
mod parser;
mod traits;

use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::{self};

pub use arg::*;
pub use directive::*;
pub use err::*;
pub use lexer::*;
pub use parser::*;

/// A runtime value that can be stored in the template context.
///
/// Values can be strings, integers, floats, or booleans. The type system
/// automatically handles conversions where appropriate (e.g., converting
/// integers to strings for display).
///
/// # Examples
///
/// ```rust
/// use figura::Value;
///
/// let s = Value::static_str("hello");
/// let i = Value::Int(42);
/// let f = Value::Float(3.14);
/// let b = Value::Bool(true);
/// ```
#[derive(Debug, Clone)]
pub enum Value {
    /// A string value (can be borrowed or owned)
    Str(Cow<'static, str>),
    /// A 64-bit signed integer
    Int(i64),
    /// A 64-bit floating point number
    Float(f64),
    /// A boolean value
    Bool(bool),
}

impl Value {
    /// Create a static string value (zero-cost)
    pub fn static_str(s: &'static str) -> Self {
        Self::Str(Cow::Borrowed(s))
    }

    /// Create an owned string value
    pub fn owned_str(s: String) -> Self {
        Self::Str(Cow::Owned(s))
    }

    /// Returns a human-readable name for the value's type.
    ///
    /// Used primarily in error messages to indicate type mismatches.
    pub fn type_name(&self) -> &str {
        match self {
            Self::Str(_) => "string",
            Self::Int(_) => "integer",
            Self::Float(_) => "float",
            Self::Bool(_) => "boolean",
        }
    }
}

/// The context passed to templates during rendering.
///
/// Maps variable names to their runtime values. Variable names must be
/// static strings for zero-copy efficiency.
///
/// # Examples
///
/// ```rust
/// use figura::{Context, Value};
/// use std::collections::HashMap;
///
/// let mut ctx = HashMap::new();
/// ctx.insert("user", Value::static_str("Alice"));
/// ctx.insert("age", Value::Int(30));
/// ```

#[cfg(feature = "foldhash")]
pub type Context = foldhash::HashMap<&'static str, Value>;

#[cfg(not(feature = "foldhash"))]
pub type Context = std::collections::HashMap<&'static str, Value>;

/// A compiled template ready for rendering.
///
/// Templates are parameterized by two characters representing the opening (`O`)
/// and closing (`C`) delimiters. Common choices are `{`/`}` or `<`/`>`.
///
/// Templates are compiled once and can be rendered multiple times with different
/// contexts, making them efficient for repeated use.
///
/// # Type Parameters
///
/// * `O` - The opening delimiter character (e.g., `'{'`)
/// * `C` - The closing delimiter character (e.g., `'}'`)
///
/// # Examples
///
/// ```rust
/// use figura::{Template, Context, Value};
/// use std::collections::HashMap;
///
/// // Using default delimiters
/// let tmpl = Template::<'{', '}'>::compile("Hello {name}!").unwrap();
///
/// let mut ctx = HashMap::new();
/// ctx.insert("name", Value::static_str("World"));
///
/// assert_eq!(tmpl.format(&ctx).unwrap(), "Hello World!");
/// ```
pub struct Template<const O: char, const C: char> {
    directives: Vec<Box<dyn Directive + Send + Sync>>,
}

impl<const C: char, const O: char> fmt::Debug for Template<O, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Template<'{}', '{}'>", O, C)
    }
}

impl<const O: char, const C: char> Template<O, C> {
    /// Compiles a template string using the default parser.
    ///
    /// This is the most common way to create a template. It uses the `DefaultParser`
    /// which supports variable substitution, repeating patterns, and conditional logic.
    ///
    /// # Arguments
    ///
    /// * `input` - The template string to compile
    ///
    /// # Returns
    ///
    /// * `Ok(Template)` - A compiled template ready for rendering
    /// * `Err(TemplateError)` - If the template syntax is invalid
    ///
    /// # Errors
    ///
    /// Returns a `TemplateError` if:
    /// - A delimiter is not properly closed
    /// - A directive cannot be parsed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use figura::{Template, Context, Value};
    /// use std::collections::HashMap;
    ///
    /// // Variable substitution
    /// let tmpl = Template::<'{', '}'>::compile("Hello {name}!").unwrap();
    ///
    /// let mut ctx = HashMap::new();
    /// ctx.insert("name", Value::static_str("World"));
    /// assert_eq!(tmpl.format(&ctx).unwrap(), "Hello World!");
    ///
    /// // Repeating patterns
    /// let tmpl = Template::<'{', '}'>::compile("{'*':count}").unwrap();
    /// let mut ctx = HashMap::new();
    /// ctx.insert("count", Value::Int(3));
    /// assert_eq!(tmpl.format(&ctx).unwrap(), "***");
    ///
    /// // Conditionals
    /// let tmpl = Template::<'{', '}'>::compile("{x > 5 ? 'big' : 'small'}").unwrap();
    /// let mut ctx = HashMap::new();
    /// ctx.insert("x", Value::Int(10));
    /// assert_eq!(tmpl.format(&ctx).unwrap(), "big");
    /// ```
    pub fn compile(input: impl AsRef<str>) -> Result<Self, TemplateError> {
        Self::compile_with_parser::<DefaultParser>(input.as_ref())
    }

    /// Compiles a template string using a custom parser.
    ///
    /// This method allows you to use a custom parser implementation for specialized
    /// template syntax or custom directives. The parser must implement the `Parser` trait.
    ///
    /// # Type Parameters
    ///
    /// * `P` - A type implementing the `Parser` trait
    ///
    /// # Arguments
    ///
    /// * `input` - The template string to compile
    ///
    /// # Returns
    ///
    /// * `Ok(Template)` - A compiled template ready for rendering
    /// * `Err(TemplateError)` - If the template syntax is invalid
    ///
    /// # Errors
    ///
    /// Returns a `TemplateError` if:
    /// - A delimiter is not properly closed
    /// - The custom parser cannot parse a directive
    ///
    /// # Examples
    ///
    /// ```rust
    /// use figura::{Template, DefaultParser, Context, Value};
    /// use std::collections::HashMap;
    ///
    /// // Using the default parser explicitly
    /// let tmpl = Template::<'{', '}'>::compile_with_parser::<DefaultParser>(
    ///     "Hello {name}!"
    /// ).unwrap();
    ///
    /// let mut ctx = HashMap::new();
    /// ctx.insert("name", Value::static_str("Alice"));
    /// assert_eq!(tmpl.format(&ctx).unwrap(), "Hello Alice!");
    /// ```
    ///
    /// For custom parser implementations, implement the `Parser` trait:
    ///
    /// ```rust
    /// use figura::{Parser, Token, Directive};
    ///
    /// struct MyCustomParser;
    ///
    /// impl Parser for MyCustomParser {
    ///     fn parse(tokens: &[Token]) -> Option<Box<dyn Directive>> {
    ///         // Your custom parsing logic here
    ///         None
    ///     }
    /// }
    /// ```
    pub fn compile_with_parser<P: Parser>(input: &str) -> Result<Self, TemplateError> {
        let mut directives: Vec<Box<dyn Directive + Send + Sync>> = Vec::new();
        let mut cursor = 0;
        let mut chars = input.char_indices().peekable();

        let arena = RefCell::new(String::new());

        while let Some((idx, ch)) = chars.next() {
            if ch == O {
                // Handle escaped opening delimiter (e.g. "{{")
                if let Some(&(_, next_char)) = chars.peek()
                    && next_char == O
                {
                    if idx > cursor {
                        directives.push(Box::new(LiteralDirective(Cow::Owned(
                            input[cursor..idx].to_string(),
                        ))));
                    }

                    directives.push(Box::new(LiteralDirective(Cow::Owned(O.to_string()))));
                    chars.next();
                    cursor = chars.peek().map(|(i, _)| *i).unwrap_or(input.len());
                    continue;
                }

                if idx > cursor {
                    directives.push(Box::new(LiteralDirective(Cow::Owned(
                        input[cursor..idx].to_string(),
                    ))));
                }

                let start = idx + ch.len_utf8();
                let mut depth = 1;
                let mut end = start;
                let mut found_close = false;

                for (c_idx, c_char) in chars.by_ref() {
                    if O == C {
                        if c_char == C {
                            depth -= 1;
                        }
                    } else if c_char == O {
                        depth += 1;
                    } else if c_char == C {
                        depth -= 1;
                    }

                    if depth == 0 {
                        end = c_idx;
                        found_close = true;
                        cursor = c_idx + C.len_utf8();
                        break;
                    }
                }

                if !found_close {
                    return Err(TemplateError::MissingDelimiter(C));
                }

                let content = &input[start..end];

                arena.borrow_mut().clear();

                let tokens: Vec<Token> = TemplateLexer::new(content).collect();

                match P::parse(&tokens) {
                    Some(directive) => directives.push(directive),
                    None => return Err(TemplateError::DirectiveParsing(content.to_string())),
                }
            } else if ch == C
                && let Some(&(_, next_char)) = chars.peek()
                && next_char == C
            {
                if idx > cursor {
                    directives.push(Box::new(LiteralDirective(Cow::Owned(
                        input[cursor..idx].to_string(),
                    ))));
                }

                directives.push(Box::new(LiteralDirective(Cow::Owned(C.to_string()))));
                chars.next();
                cursor = chars.peek().map(|(i, _)| *i).unwrap_or(input.len());
                continue;
            }
        }

        if cursor < input.len() {
            directives.push(Box::new(LiteralDirective(Cow::Owned(
                input[cursor..].to_string(),
            ))));
        }

        Ok(Self { directives })
    }

    /// Renders the template using the provided context.
    ///
    /// This method executes all directives in the template and concatenates their
    /// results into a final string. It pre-allocates a reasonable capacity to minimize
    /// allocations during rendering.
    ///
    /// # Arguments
    ///
    /// * `ctx` - A reference to the context containing variable values
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The rendered template output
    /// * `Err(DirectiveError)` - If any directive fails (e.g., missing variable, type mismatch)
    ///
    /// # Errors
    ///
    /// Returns a `DirectiveError` if:
    /// - A referenced variable is not found in the context
    /// - A variable has an incompatible type for the operation
    /// - A literal value cannot be parsed as the required type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use figura::{Template, Context, Value};
    /// use std::collections::HashMap;
    ///
    /// let tmpl = Template::<'{', '}'>::compile("Hi {name}!").unwrap();
    ///
    /// let mut ctx = HashMap::new();
    /// ctx.insert("name", Value::static_str("Alice"));
    ///
    /// let output = tmpl.format(&ctx).unwrap();
    /// assert_eq!(output, "Hi Alice!");
    /// ```
    pub fn format(&self, ctx: &Context) -> Result<String, DirectiveError> {
        let mut output = String::with_capacity(self.directives.len() * 8);

        for directive in &self.directives {
            let result = directive.exec(ctx)?;
            output.push_str(&result);
        }

        Ok(output)
    }
}
