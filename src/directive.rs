use std::borrow::Cow;

use crate::Context;
use crate::arg::Argument;
use crate::err::DirectiveError;

/// A template directive that can be executed to produce output.
///
/// Directives are the executable components of a compiled template. Each directive
/// represents a unit of work that can be performed during template rendering, such as:
/// - Outputting literal text
/// - Substituting variable values
/// - Repeating patterns
/// - Conditional branching
///
/// Directives are trait objects stored in the compiled template and executed
/// sequentially during the `format` operation.
pub trait Directive {
    /// Executes this directive with the given context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The template context containing variable values
    ///
    /// # Returns
    ///
    /// * `Ok(Cow<'static, str>)` - The output string produced by this directive
    /// * `Err(DirectiveError)` - If execution fails (e.g., missing variable)
    ///
    /// # Errors
    ///
    /// Returns an error if the directive cannot be executed, such as when
    /// a required variable is missing or has an incompatible type.
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, DirectiveError>;
}

/// A directive that produces no output.
///
/// Used as a placeholder when parsing encounters an empty or invalid expression
/// that should be silently ignored rather than causing a compilation error.
///
/// # Examples
///
/// An empty directive always returns an empty string regardless of context.
pub struct EmptyDirective;

impl Directive for EmptyDirective {
    fn exec(&self, _ctx: &Context) -> Result<Cow<'static, str>, DirectiveError> {
        Ok(Cow::Borrowed(""))
    }
}

/// A directive that outputs a literal string.
///
/// This is used for the static portions of a template that don't involve
/// any variable substitution or dynamic behavior. The string is stored
/// as a `Cow` to enable zero-copy when possible.
///
/// # Examples
///
/// ```text
/// Template: "Hello {name}!"
/// Produces three directives:
///   1. LiteralDirective("Hello ")
///   2. ReplaceDirective(name)
///   3. LiteralDirective("!")
/// ```
pub struct LiteralDirective(pub Cow<'static, str>);

impl Directive for LiteralDirective {
    fn exec(&self, _ctx: &Context) -> Result<Cow<'static, str>, DirectiveError> {
        Ok(self.0.clone())
    }
}

/// A directive that substitutes a variable or evaluates an expression.
///
/// This is the most common directive type, used for simple variable replacement
/// like `{name}` or literal values like `{"hello"}`.
///
/// # Examples
///
/// ```text
/// Template: "{username}"
/// With context: username = "Alice"
/// Produces: "Alice"
/// ```
///
/// # Errors
///
/// Returns an error if the argument cannot be resolved (e.g., variable not found).
pub struct ReplaceDirective(pub Argument);

impl Directive for ReplaceDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, DirectiveError> {
        self.0.resolve_as::<Cow<'static, str>>(ctx)
    }
}

/// A directive that repeats a pattern a specified number of times.
///
/// Syntax: `{pattern:count}` where:
/// - `pattern` is the string to repeat (variable or literal)
/// - `count` is the number of repetitions (variable or literal integer)
///
/// # Examples
///
/// ```text
/// Template: "{'*':3}"
/// Produces: "***"
///
/// Template: "{char:n}"
/// With context: char = "-", n = 5
/// Produces: "-----"
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The pattern argument cannot be resolved to a string
/// - The count argument cannot be resolved to an integer
pub struct RepeatDirective(pub Argument, pub Argument);

impl Directive for RepeatDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, DirectiveError> {
        let pattern = self.0.resolve_as::<Cow<'static, str>>(ctx)?;
        let count = self.1.resolve_as::<i64>(ctx)?;

        Ok(Cow::Owned(pattern.repeat(count as usize)))
    }
}

/// A directive that performs conditional branching (ternary operator).
///
/// Evaluates a condition and returns one of two values based on the result.
/// Syntax: `{condition ? true_value : false_value}`
///
/// The condition can be:
/// - A boolean variable
/// - A boolean literal
/// - A comparison expression (e.g., `x == 5`, `a > b`)
/// - A NOT expression (e.g., `!active`)
///
/// # Examples
///
/// ```text
/// Template: "{active ? 'yes' : 'no'}"
/// With context: active = true
/// Produces: "yes"
///
/// Template: "{count > 0 ? 'items' : 'empty'}"
/// With context: count = 5
/// Produces: "items"
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The condition cannot be resolved to a boolean
/// - The selected branch argument cannot be resolved
pub struct ConditionalDirective {
    /// The condition to evaluate
    pub cond: Argument,
    /// The value to return if the condition is true
    pub if_true: Argument,
    /// The value to return if the condition is false
    pub if_false: Argument,
}

impl Directive for ConditionalDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, DirectiveError> {
        let cond_value = self.cond.resolve_as::<bool>(ctx)?;

        if cond_value {
            self.if_true.resolve_as::<Cow<'static, str>>(ctx)
        } else {
            self.if_false.resolve_as::<Cow<'static, str>>(ctx)
        }
    }
}
