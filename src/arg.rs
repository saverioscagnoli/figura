use std::borrow::Cow;

use crate::Context;
use crate::Value;
use crate::err::DirectiveError;
use crate::traits::ToAstring;

/// An argument that can be resolved to a value at runtime.
///
/// Arguments are the building blocks of template expressions. They can represent:
/// - **Variables**: Values looked up from the context by name (e.g., `{name}`)
/// - **Literals**: Constant values embedded in the template (e.g., `{"hello"}`)
/// - **Expressions**: Complex expressions that evaluate to values (e.g., comparisons)
///
/// # Examples
///
/// ```rust
/// use figura::Argument;
/// use std::borrow::Cow;
///
/// // Create a variable reference
/// let var = Argument::variable(Cow::Borrowed("username"));
///
/// // Create a literal value
/// let lit = Argument::literal(Cow::Borrowed("42"));
/// ```
#[derive(Debug, Clone)]
pub enum Argument {
    /// A variable name to be looked up in the context.
    ///
    /// When resolved, the value is retrieved from the template context
    /// using this name as the key.
    Variable(Cow<'static, str>),

    /// A literal value embedded directly in the template.
    ///
    /// This value is used as-is and parsed into the required type
    /// when the argument is resolved.
    Literal(Cow<'static, str>),

    /// A complex expression that evaluates to a value.
    ///
    /// Expressions include comparisons and logical operations that
    /// compute a result based on other arguments.
    Expression(Box<Expression>),
}

/// Comparison operators for use in conditional expressions.
///
/// These operators compare two values and produce a boolean result.
/// Numeric comparisons are performed when both operands can be parsed as numbers;
/// otherwise, string comparison is used.
#[derive(Debug, Clone)]
pub enum ComparisonOp {
    /// Equality: `==`
    Equals,
    /// Inequality: `!=`
    NotEquals,
    /// Greater than: `>`
    GreaterThan,
    /// Less than: `<`
    LessThan,
    /// Greater than or equal: `>=`
    GreaterThanEquals,
    /// Less than or equal: `<=`
    LessThanEquals,
}

/// An expression that can be evaluated to produce a value.
///
/// Expressions support comparison operations and logical negation.
/// They are typically used in conditional directives to determine
/// which branch to take.
///
/// # Examples
///
/// ```text
/// {x == 5 ? "yes" : "no"}      // Comparison expression
/// {!active ? "inactive" : "active"}  // NOT expression
/// ```
#[derive(Debug, Clone)]
pub enum Expression {
    /// A binary comparison between two arguments.
    ///
    /// Evaluates to a boolean by comparing the left and right arguments
    /// using the specified operator. Supports both numeric and string comparisons.
    Comparison {
        /// Left-hand side of the comparison
        left: Argument,
        /// The comparison operator
        op: ComparisonOp,
        /// Right-hand side of the comparison
        right: Argument,
    },
    /// Logical NOT operation.
    ///
    /// Negates the boolean value of the argument. The argument must
    /// resolve to a boolean or truthy/falsy value.
    Not(Argument),
}

impl Argument {
    /// Creates a variable argument that references a context value.
    ///
    /// # Arguments
    ///
    /// * `name` - The variable name to look up in the context
    pub fn variable(name: Cow<'static, str>) -> Self {
        Self::Variable(name)
    }

    /// Creates a literal argument with a constant value.
    ///
    /// # Arguments
    ///
    /// * `s` - The literal string value
    pub fn literal(s: Cow<'static, str>) -> Self {
        Self::Literal(s)
    }

    /// Creates an expression argument from an Expression.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to wrap
    pub fn expression(expr: Expression) -> Self {
        Self::Expression(Box::new(expr))
    }

    /// Creates a comparison expression argument.
    ///
    /// This is a convenience method for creating comparison expressions
    /// without manually constructing the Expression enum.
    ///
    /// # Arguments
    ///
    /// * `left` - Left-hand side argument
    /// * `op` - Comparison operator
    /// * `right` - Right-hand side argument
    pub fn comparison(left: Self, op: ComparisonOp, right: Self) -> Self {
        Self::Expression(Box::new(Expression::Comparison { left, op, right }))
    }

    /// Creates a NOT expression argument.
    ///
    /// This is a convenience method for negating an argument's boolean value.
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument to negate
    pub fn not(arg: Self) -> Self {
        Self::Expression(Box::new(Expression::Not(arg)))
    }
}

/// Types that can be resolved from template arguments.
///
/// This trait enables converting template values (from context or literals)
/// into strongly-typed Rust values. It handles both direct value conversions
/// and parsing from string representations.
///
/// # Type Parameters
///
/// Implementors must provide:
/// - `TYPE_NAME`: A human-readable name for error messages
/// - `from_value`: Convert from a runtime `Value`
/// - `from_string_slice`: Parse from a string literal
///
/// # Implementations
///
/// Built-in implementations exist for:
/// - `Cow<'static, str>` (strings)
/// - `i64` (integers)
/// - `f64` (floats)
/// - `bool` (booleans)
pub trait Resolvable: Sized {
    /// The human-readable name of this type, used in error messages.
    const TYPE_NAME: &'static str;

    /// Attempts to convert a runtime Value into this type.
    ///
    /// # Arguments
    ///
    /// * `value` - The runtime value to convert
    ///
    /// # Returns
    ///
    /// `Some(Self)` if conversion succeeds, `None` otherwise.
    fn from_value(value: &Value) -> Option<Self>;

    /// Attempts to parse a string literal into this type.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse
    ///
    /// # Returns
    ///
    /// `Ok(Self)` if parsing succeeds, `Err(String)` with an error message otherwise.
    fn from_string_slice(s: &str) -> Result<Self, String>;
}

impl Argument {
    /// Resolves this argument to a concrete value of type `T`.
    ///
    /// This method handles the logic of:
    /// - Looking up variables in the context
    /// - Parsing literal values
    /// - Evaluating expressions
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to resolve to (must implement `Resolvable`)
    ///
    /// # Arguments
    ///
    /// * `ctx` - The template context containing variable values
    ///
    /// # Returns
    ///
    /// * `Ok(T)` - Successfully resolved value
    /// * `Err(DirectiveError)` - Resolution failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Variable is not found in the context (`DirectiveError::NotFound`)
    /// - Variable has wrong type (`DirectiveError::TypeError`)
    /// - Literal cannot be parsed (`DirectiveError::ParseError`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use figura::{Argument, Context, Value};
    /// use std::borrow::Cow;
    /// use std::collections::HashMap;
    ///
    /// let mut ctx = HashMap::new();
    /// ctx.insert("count", Value::Int(42));
    ///
    /// let arg = Argument::variable(Cow::Borrowed("count"));
    /// let value: i64 = arg.resolve_as(&ctx).unwrap();
    /// assert_eq!(value, 42);
    /// ```
    pub fn resolve_as<T: Resolvable>(&self, ctx: &Context) -> Result<T, DirectiveError> {
        match self {
            Self::Variable(name) => {
                if let Some(value) = ctx.get(name.as_ref()) {
                    if let Some(parsed) = T::from_value(value) {
                        return Ok(parsed);
                    }

                    // Variable is of the wrong type but was found.
                    return Err(DirectiveError::TypeError {
                        name: name.to_string(),
                        expected: T::TYPE_NAME,
                        found: value.type_name().to_string(),
                    });
                }

                // Not in the context
                Err(DirectiveError::NotFound {
                    name: name.to_string(),
                    type_name: T::TYPE_NAME,
                })
            }

            Self::Literal(value) => {
                // Just try to parse it
                T::from_string_slice(value).map_err(|err| DirectiveError::ParseError {
                    value: value.to_string(),
                    type_name: T::TYPE_NAME,
                    message: err,
                })
            }

            Self::Expression(expr) => {
                // Evaluate the expression and convert to the requested type
                let result = expr.evaluate(ctx)?;

                T::from_value(&result).ok_or_else(|| DirectiveError::TypeError {
                    name: "expression".to_string(),
                    expected: T::TYPE_NAME,
                    found: result.type_name().to_string(),
                })
            }
        }
    }
}

impl Expression {
    /// Evaluates this expression to produce a runtime value.
    ///
    /// Comparison expressions attempt numeric comparison when both sides
    /// can be parsed as floats; otherwise, they fall back to string comparison.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The template context for resolving variables
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - The result of evaluating the expression
    /// * `Err(DirectiveError)` - Evaluation failed
    ///
    /// # Errors
    ///
    /// Returns an error if any sub-argument fails to resolve.
    pub fn evaluate(&self, ctx: &Context) -> Result<crate::Value, DirectiveError> {
        match self {
            Self::Comparison { left, op, right } => {
                // Try to resolve both sides as strings first, then try numeric comparison
                let left_str = left.resolve_as::<Cow<'static, str>>(ctx)?;
                let right_str = right.resolve_as::<Cow<'static, str>>(ctx)?;

                let result = match op {
                    ComparisonOp::Equals => left_str == right_str,
                    ComparisonOp::NotEquals => left_str != right_str,
                    ComparisonOp::GreaterThan => {
                        // Try numeric comparison
                        // Treating anything as a float
                        if let (Ok(l), Ok(r)) = (left_str.parse::<f64>(), right_str.parse::<f64>())
                        {
                            l > r
                        } else {
                            left_str > right_str
                        }
                    }
                    ComparisonOp::LessThan => {
                        if let (Ok(l), Ok(r)) = (left_str.parse::<f64>(), right_str.parse::<f64>())
                        {
                            l < r
                        } else {
                            left_str < right_str
                        }
                    }
                    ComparisonOp::GreaterThanEquals => {
                        if let (Ok(l), Ok(r)) = (left_str.parse::<f64>(), right_str.parse::<f64>())
                        {
                            l >= r
                        } else {
                            left_str >= right_str
                        }
                    }
                    ComparisonOp::LessThanEquals => {
                        if let (Ok(l), Ok(r)) = (left_str.parse::<f64>(), right_str.parse::<f64>())
                        {
                            l <= r
                        } else {
                            left_str <= right_str
                        }
                    }
                };

                Ok(Value::Bool(result))
            }
            Self::Not(arg) => {
                let value = arg.resolve_as::<bool>(ctx)?;

                Ok(Value::Bool(!value))
            }
        }
    }
}

impl Resolvable for Cow<'static, str> {
    const TYPE_NAME: &'static str = "string";

    /// Converts any Value type to a string representation.
    ///
    /// All value types can be converted to strings, making this
    /// conversion infallible.
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(v) => Some(v.clone()),
            Value::Int(v) => Some(Cow::Owned(v.to_astring())),
            Value::Float(v) => Some(Cow::Owned(v.to_astring())),
            Value::Bool(v) => Some(Cow::Owned(v.to_string())),
        }
    }

    fn from_string_slice(s: &str) -> Result<Self, String> {
        Ok(Cow::Owned(s.to_string()))
    }
}

impl Resolvable for i64 {
    const TYPE_NAME: &'static str = "i64";

    /// Converts a Value to a 64-bit integer.
    ///
    /// - Strings are parsed
    /// - Floats are truncated
    /// - Booleans become 0 or 1
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(v) => v.parse().ok(),

            Value::Int(v) => Some(*v),
            Value::Float(v) => Some(*v as Self),
            Value::Bool(v) => Some(*v as Self),
        }
    }

    fn from_string_slice(s: &str) -> Result<Self, String> {
        s.parse::<Self>().map_err(|e| e.to_string())
    }
}

impl Resolvable for f64 {
    const TYPE_NAME: &'static str = "float";

    /// Converts a Value to a 64-bit float.
    ///
    /// - Strings are parsed
    /// - Integers are converted
    /// - Booleans become 0.0 or 1.0
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(v) => v.parse().ok(),

            Value::Int(v) => Some(*v as Self),
            Value::Float(v) => Some(*v),
            Value::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
        }
    }

    fn from_string_slice(s: &str) -> Result<Self, String> {
        s.parse::<Self>().map_err(|e| e.to_string())
    }
}

impl Resolvable for bool {
    const TYPE_NAME: &'static str = "bool";

    /// Converts a Value to a boolean.
    ///
    /// - Strings are parsed ("true"/"false")
    /// - Integers use zero/non-zero semantics
    /// - Floats use zero/non-zero semantics
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(v) => v.parse().ok(),

            Value::Int(v) => Some(*v != 0),
            Value::Float(v) => Some(*v != 0.0),
            Value::Bool(v) => Some(*v),
        }
    }

    fn from_string_slice(s: &str) -> Result<Self, String> {
        s.parse::<Self>().map_err(|e| e.to_string())
    }
}
