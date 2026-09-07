//! Parsing template expressions into executable directives.
//!
//! This module provides the parsing layer that converts tokenized template
//! expressions into executable directives. The parser uses pattern matching
//! on token sequences to recognize different template syntaxes.

use std::borrow::Cow;

use crate::arg::Argument;
use crate::arg::ComparisonOp;
use crate::directive::ConditionalDirective;
use crate::directive::Directive;
use crate::directive::EmptyDirective;
use crate::directive::RepeatDirective;
use crate::directive::ReplaceDirective;
use crate::lexer::Token;

/// A parser that converts token sequences into executable directives.
///
/// Parsers implement the logic for recognizing different template expression
/// patterns and building the corresponding directive objects. Different parser
/// implementations can support different template syntaxes.
///
/// Note: This parser never return `None`, it just returns an empty directive which does nothring.
///
/// # Examples
///
/// The default parser recognizes patterns like:
/// - `{name}` → Variable replacement
/// - `{pattern:count}` → Repeat directive
/// - `{cond ? true : false}` → Conditional directive
///
/// Custom parsers can be implemented to support alternative syntaxes or
/// additional features.
pub trait Parser {
    /// Parses a token sequence into a directive.
    ///
    /// # Arguments
    ///
    /// * `tokens` - A slice of tokens to parse
    ///
    /// # Returns
    ///
    /// * `Some(Box<dyn Directive>)` - Successfully parsed directive
    /// * `None` - Parse failed (invalid syntax)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use figura::{DefaultParser, Parser, TemplateLexer};
    ///
    /// let tokens: Vec<_> = TemplateLexer::new("name").collect();
    /// let directive = DefaultParser::parse(&tokens);
    /// assert!(directive.is_some());
    /// ```
    fn parse(tokens: &[Token]) -> Option<Box<dyn Directive + Send + Sync>>;
}

/// The default parser implementation.
///
/// Supports the standard Figura template syntax including:
/// - **Variable substitution**: `{name}` - Replaces with context value
/// - **Literal values**: `{"text"}` or `{42}` - Uses literal values
/// - **Repeat patterns**: `{pattern:count}` - Repeats pattern N times
/// - **Simple conditionals**: `{condition ? true_value : false_value}`
/// - **Comparison conditionals**: `{x == 5 ? "yes" : "no"}`
/// - **Logical NOT**: `{!active ? "inactive" : "active"}`
///
/// Supported comparison operators: `==`, `!=`, `>`, `<`, `>=`, `<=`
///
/// # Examples
///
/// ```rust
/// use figura::{Template, DefaultParser};
///
/// // Variable substitution
/// let t1 = Template::<'{', '}'>::compile::<DefaultParser>("{name}").unwrap();
///
/// // Repeat pattern
/// let t2 = Template::<'{', '}'>::compile::<DefaultParser>("{'*':3}").unwrap();
///
/// // Conditional with comparison
/// let t3 = Template::<'{', '}'>::compile::<DefaultParser>(
///     "{age >= 18 ? 'adult' : 'minor'}"
/// ).unwrap();
/// ```
pub struct DefaultParser;

/// Converts a token into an argument for use in directives.
///
/// # Arguments
///
/// * `token` - The token to convert
///
/// # Returns
///
/// An `Argument` representing the token's value:
/// - `Ident` → Variable argument
/// - `Literal`, `Int`, `Float` → Literal argument
/// - Other tokens → Empty literal
fn token_to_argument(token: &Token) -> Argument {
    match token {
        Token::Ident(s) => Argument::variable(Cow::Owned(s.to_string())),
        Token::Literal(s) => Argument::literal(Cow::Owned(s.to_string())),
        Token::Int(s) => Argument::literal(Cow::Owned(s.to_string())),
        Token::Float(s) => Argument::literal(Cow::Owned(s.to_string())),
        _ => Argument::literal(Cow::Borrowed("")),
    }
}

impl Parser for DefaultParser {
    /// Parses tokens into directives using pattern matching.
    ///
    /// This implementation matches token sequences against known patterns
    /// and constructs the appropriate directive type. Patterns are matched
    /// in order, with more specific patterns checked first.
    ///
    /// # Supported Patterns
    ///
    /// 1. **Variable replacement**: `[Ident]` → `{name}`
    /// 2. **Literal value**: `[Literal]` → `{"text"}`
    /// 3. **Repeat pattern**: `[Pattern, Colon, Count]` → `{pattern:count}`
    /// 4. **Simple conditional**: `[Cond, Question, True, Colon, False]` → `{cond ? true : false}`
    /// 5. **Comparison conditional**: `[Left, Op, Right, Question, True, Colon, False]` → `{x == 5 ? yes : no}`
    /// 6. **NOT conditional**: `[Not, Cond, Question, True, Colon, False]` → `{!cond ? yes : no}`
    ///
    /// # Returns
    ///
    /// Returns `Some(directive)` if parsing succeeds, or `Some(EmptyDirective)` if
    /// the token sequence doesn't match any known pattern. Returns `None` only if
    /// a critical parsing error occurs (currently never happens in practice).
    fn parse(tokens: &[Token]) -> Option<Box<dyn Directive + Send + Sync>> {
        match tokens {
            // Simple variable replacement: {name}
            // Example: {username} → ReplaceDirective(Variable("username"))
            [Token::Ident(ident)] => Some(Box::new(ReplaceDirective(Argument::variable(
                Cow::Owned(ident.to_string()),
            )))),

            // Repeat pattern: {pattern:count}
            // Examples:
            //   {'*':3} → RepeatDirective(Literal("*"), Literal("3"))
            //   {char:n} → RepeatDirective(Variable("char"), Variable("n"))
            [
                p @ (Token::Ident(_) | Token::Literal(_)),
                Token::Colon,
                c @ (Token::Ident(_) | Token::Int(_)),
            ] => {
                let pattern = match p {
                    Token::Ident(s) => Argument::variable(Cow::Owned(s.to_string())),
                    Token::Literal(cow) => Argument::literal(Cow::Owned(cow.to_string())),
                    _ => unreachable!(),
                };

                let count = match c {
                    Token::Ident(s) => Argument::variable(Cow::Owned(s.to_string())),
                    Token::Int(s) => Argument::literal(Cow::Owned(s.to_string())),
                    _ => unreachable!(),
                };

                Some(Box::new(RepeatDirective(pattern, count)))
            }

            // Literal replacement: {"text"}
            // Example: {"hello"} → ReplaceDirective(Literal("hello"))
            [Token::Literal(lit)] => Some(Box::new(ReplaceDirective(Argument::literal(
                Cow::Owned(lit.to_string()),
            )))),

            // Ternary conditional: {condition ? if_true : if_false}
            // The condition can be a variable, literal, or numeric value
            // Examples:
            //   {active ? "yes" : "no"} → ConditionalDirective with Variable("active")
            //   {true ? "yes" : "no"} → ConditionalDirective with Literal("true")
            [
                cond @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_)),
                Token::Question,
                if_true @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_)),
                Token::Colon,
                if_false @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_)),
            ] => {
                let cond_arg = token_to_argument(cond);
                let true_arg = token_to_argument(if_true);
                let false_arg = token_to_argument(if_false);

                Some(Box::new(ConditionalDirective {
                    cond: cond_arg,
                    if_true: true_arg,
                    if_false: false_arg,
                }))
            }

            // Conditional with comparison: {left op right ? if_true : if_false}
            // Supports: ==, !=, >, <, >=, <=
            // Examples:
            //   {age >= 18 ? "adult" : "minor"}
            //   {status == "active" ? "online" : "offline"}
            //   {count > 0 ? items : "empty"}
            [
                left @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_) | Token::Float(_)),
                op @ (Token::Equals
                | Token::NotEquals
                | Token::GreaterThan
                | Token::LessThan
                | Token::GreaterThanEquals
                | Token::LessThanEquals),
                right @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_) | Token::Float(_)),
                Token::Question,
                if_true @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_)),
                Token::Colon,
                if_false @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_)),
            ] => {
                let left_arg = token_to_argument(left);
                let right_arg = token_to_argument(right);

                let comp_op = match op {
                    Token::Equals => ComparisonOp::Equals,
                    Token::NotEquals => ComparisonOp::NotEquals,
                    Token::GreaterThan => ComparisonOp::GreaterThan,
                    Token::LessThan => ComparisonOp::LessThan,
                    Token::GreaterThanEquals => ComparisonOp::GreaterThanEquals,
                    Token::LessThanEquals => ComparisonOp::LessThanEquals,
                    _ => unreachable!(),
                };

                let cond_arg = Argument::comparison(left_arg, comp_op, right_arg);
                let true_arg = token_to_argument(if_true);
                let false_arg = token_to_argument(if_false);

                Some(Box::new(ConditionalDirective {
                    cond: cond_arg,
                    if_true: true_arg,
                    if_false: false_arg,
                }))
            }

            // Logical NOT conditional: {!condition ? if_true : if_false}
            // Examples:
            //   {!active ? "disabled" : "enabled"}
            //   {!0 ? "truthy" : "falsy"}
            [
                Token::Not,
                cond @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_)),
                Token::Question,
                if_true @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_)),
                Token::Colon,
                if_false @ (Token::Ident(_) | Token::Literal(_) | Token::Int(_)),
            ] => {
                let inner_arg = token_to_argument(cond);
                let cond_arg = Argument::not(inner_arg);
                let true_arg = token_to_argument(if_true);
                let false_arg = token_to_argument(if_false);

                Some(Box::new(ConditionalDirective {
                    cond: cond_arg,
                    if_true: true_arg,
                    if_false: false_arg,
                }))
            }

            // Unknown pattern: return empty directive (silent failure)
            // This allows templates to compile even with unsupported expressions,
            // which will simply produce no output rather than failing to compile.
            _ => Some(Box::new(EmptyDirective)),
        }
    }
}
