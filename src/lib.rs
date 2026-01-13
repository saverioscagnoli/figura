//! Template formatting engine core module.

mod directives;
mod error;
mod traits;

pub use directives::*;
pub use error::*;
pub use traits::*;

use smacro::s;
use std::{
    collections::HashMap,
    fmt::{Display, Write},
};

/// A simple value type used in templating context.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    /// String value.
    String(String),

    /// Integer value.
    Int(i64),

    /// Floating-point value.
    Float(f64),

    /// Boolean value.
    Bool(bool),

    /// &'static str value
    Str(&'static str),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => f.write_str(s),
            Value::Int(i) => f.write_str(&i.to_astring()),
            Value::Float(n) => f.write_str(&n.to_astring()),
            Value::Bool(b) => f.write_str(&b.to_string()),
            Value::Str(s) => f.write_str(s),
        }
    }
}

/// A key-value map representing template context data.
///
/// # Example
/// ```no_run
/// use std::collections::HashMap;
/// use your_crate::Value;
///
/// let mut ctx: Context = HashMap::new();
/// ctx.insert("name", Value::String("Alice".to_string()));
/// ```
pub type Context = HashMap<&'static str, Value>;

/// A token used in directive parsing.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    /// A string literal, which can contain alphanumeric characters, underscores, and whitespace.
    /// It can be treated as an identifier or a slice of text.
    Slice(String),
    /// A symbolic character (e.g., `:`, `+`, etc.).
    /// _ and whitespace are excluded
    Symbol(char),
    /// An integer literal.
    Int(i64),
    // Float
    Float(f64),
    /// Any unrecognized character.
    Uknown(char),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Slice(s) => f.write_str(s),
            Token::Symbol(s) => f.write_char(*s),
            Token::Int(i) => f.write_str(&i.to_astring()),
            Token::Float(n) => f.write_str(&n.to_astring()),
            Token::Uknown(c) => f.write_char(*c),
        }
    }
}

/// Specifies alignment options for formatted output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    /// Left-aligned output (`<`).
    #[default]
    Left,
    /// Right-aligned output (`>`).
    Right,
    /// Center-aligned output (`^`).
    Center,
}

impl Alignment {
    /// Parses a character into an [`Alignment`] option.
    ///
    /// Returns `None` if the character doesn't match any known alignment.
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            '<' => Some(Alignment::Left),
            '>' => Some(Alignment::Right),
            '^' => Some(Alignment::Center),
            _ => None,
        }
    }
}

/// Represents a part of a parsed template: either a literal or a directive.
#[derive(Debug)]
pub enum Part {
    /// Literal string part of the template.
    Literal(String),
    /// A directive that implements the [`Directive`] trait.
    Directive(Box<dyn Directive>),
}

/// A parsed template, parameterized by opening (`O`) and closing (`C`) delimiters.
///
/// This struct can parse a template string and then format it using a [`Context`].
///
/// # Example
///
/// ```no_run
/// let tpl = Template::parse::<DefaultParser>("Hello, {name}!")?;
/// let mut ctx = Context::new();
/// ctx.insert("name", Value::String("World".into()));
/// let result = tpl.format(&ctx)?;
/// assert_eq!(result, "Hello, World!");
/// ```
#[derive(Debug)]
pub struct Template<const O: char = '{', const C: char = '}'> {
    parts: Vec<Part>,
    alignment: Alignment,
}

impl<const O: char, const C: char> Template<O, C> {
    /// Parses a template string into a [`Template`] instance using the default parser.
    ///
    /// To use your custom parser, use [`Template::with_parser`].
    pub fn parse<T: AsRef<str>>(t: T) -> Result<Self, TemplateError> {
        Self::with_parser::<DefaultParser>(t.as_ref())
    }

    /// Parses a template string into a [`Template`] instance.
    ///
    /// Uses the provided parser `P` to handle directives.
    /// The default parse function internally uses this function like this:
    /// ```no_run
    /// Self::with_parser::<DefaultParser>(t)
    /// ```
    pub fn with_parser<P: Parser>(t: &str) -> Result<Self, TemplateError> {
        let mut chars = t.chars().peekable();

        Self::validate_delimiters(t)?;

        let mut literal = String::new();
        let mut alignment = Alignment::default();
        let mut parts = Vec::new();

        while let Some(ch) = chars.next() {
            match ch {
                c if c == O => {
                    if let Some(next_ch) = chars.peek()
                        && next_ch == &O
                    {
                        // Double opening character
                        // Escape sequence
                        chars.next();
                        literal.push(O);
                        continue;
                    }

                    // If a string literal has been accumulated,
                    // push its content and clear it
                    if !literal.is_empty() {
                        parts.push(Part::Literal(literal.clone()));
                        literal.clear();
                    }

                    // The text content in-between the delimiters
                    let mut content = String::new();

                    for next_ch in chars.by_ref() {
                        if next_ch == C {
                            break;
                        }

                        content.push(next_ch);
                    }

                    if let Some(last_ch) = content.chars().last()
                        && let Some(a) = Alignment::from_char(last_ch)
                    {
                        alignment = a;
                        // Remove the last character (alignment character) from the string
                        content = content[..content.len() - last_ch.len_utf8()].to_string();
                    }

                    let tokens = Self::tokenize(&content);

                    if let Some(d) = P::parse(&tokens, &content) {
                        parts.push(Part::Directive(d));
                    } else {
                        return Err(TemplateError::DirectiveParsing(content));
                    }
                }

                c if c == C => {
                    if let Some(next_ch) = chars.peek() {
                        if next_ch == &C {
                            // Double closing character - escape sequence
                            chars.next();
                            literal.push(C);
                        } else {
                            // Single closing character - add to literal
                            literal.push(C);
                        }
                    } else {
                        // Single closing character at end of string
                        literal.push(C);
                    }
                }

                _ => literal.push(ch),
            }
        }

        // Check for hanging literals
        if !literal.is_empty() {
            parts.push(Part::Literal(literal));
        }

        Ok(Self { parts, alignment })
    }

    /// Formats the template using the given context.
    ///
    /// Each directive is executed against the provided context map.
    pub fn format(&self, ctx: &Context) -> Result<String, TemplateError> {
        let mut result = String::new();

        for part in &self.parts {
            match part {
                Part::Literal(s) => result.push_str(s),
                Part::Directive(d) => result.push_str(&d.execute(ctx)?),
            }
        }

        Ok(result)
    }

    /// Validates the delimiters in the template string.
    ///
    /// Ensures that all opening delimiters have a matching closing one.
    fn validate_delimiters(input: &str) -> Result<(), TemplateError> {
        let mut chars = input.chars().peekable();

        if C == O {
            if chars.filter(|c| *c != C).count() % 2 != 0 {
                return Err(TemplateError::MissingDelimiter(O));
            } else {
                return Ok(());
            }
        }

        let mut depth = 0;

        while let Some(ch) = chars.next() {
            match ch {
                c if c == O => {
                    // Check for escape sequence {{
                    if chars.peek() == Some(&O) {
                        chars.next();
                    } else {
                        depth += 1;
                    }
                }
                c if c == C => {
                    // Check for escape sequence }}
                    if chars.peek() == Some(&C) {
                        chars.next();
                    } else {
                        depth -= 1;
                        if depth < 0 {
                            return Err(TemplateError::MissingOpenDelimiter(O));
                        }
                    }
                }
                _ => {}
            }
        }

        if depth > 0 {
            return Err(TemplateError::MissingClosedDelimiter(C));
        }

        Ok(())
    }

    fn flush_slice(slice: &mut String, tokens: &mut Vec<Token>) {
        if slice.is_empty() {
            return;
        }

        tokens.push(Token::Slice(s!(slice)));
        slice.clear();
    }

    fn flush_number(number: &mut String, tokens: &mut Vec<Token>, is_float: &mut bool) {
        if number.is_empty() {
            return;
        }

        if *is_float {
            tokens.push(Token::Float(number.parse::<f64>().unwrap()));
            *is_float = false;
        } else {
            tokens.push(Token::Int(number.parse::<i64>().unwrap()));
        }

        number.clear();
    }

    /// Function to convert a section between delimiters
    /// into a vector of token, which will be passed into
    /// the parse trait function so that the user can customize the logic
    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();
        // Keeps track of the current string literal
        let mut slice = String::new();
        // Keeps track of the current number (for multi-digits)
        let mut tracking_float = false;
        let mut number = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                // Slice token
                c if c.is_alphabetic() || c == '_' || c.is_whitespace() => {
                    Self::flush_number(&mut number, &mut tokens, &mut tracking_float);

                    slice.push(c)
                }

                // Number token
                c if c.is_ascii_digit() => {
                    Self::flush_slice(&mut slice, &mut tokens);

                    number.push(c);
                }

                // '.', check for potential float
                c if c == '.' => {
                    Self::flush_slice(&mut slice, &mut tokens);

                    if !number.is_empty() {
                        number.push(c);
                        tracking_float = true;
                    } else {
                        // Peek, if the next character is a digit, it is a float (i.e., .3)
                        if let Some(next_ch) = chars.peek() {
                            if next_ch.is_ascii_digit() {
                                number.push(c);
                                tracking_float = true;
                            } else {
                                Self::flush_number(&mut number, &mut tokens, &mut tracking_float);
                                tokens.push(Token::Symbol(c));
                            }
                        } else {
                            // If no next character, treat it as a symbol
                            Self::flush_number(&mut number, &mut tokens, &mut tracking_float);
                            tokens.push(Token::Symbol(c));
                        }
                    }
                }

                // '-', check for potential negative number
                c if c == '-' => {
                    Self::flush_slice(&mut slice, &mut tokens);
                    Self::flush_number(&mut number, &mut tokens, &mut tracking_float);

                    // If the next character is a digit, treat it as a negative number
                    if let Some(next_ch) = chars.peek() {
                        if next_ch.is_ascii_digit() {
                            number.push(c);
                        } else {
                            tokens.push(Token::Symbol(c));
                        }
                    } else {
                        tokens.push(Token::Symbol(c));
                    }
                }

                // Symbol token
                c if !c.is_alphanumeric() && !ch.is_whitespace() => {
                    Self::flush_slice(&mut slice, &mut tokens);
                    Self::flush_number(&mut number, &mut tokens, &mut tracking_float);

                    tokens.push(Token::Symbol(c));
                }

                c => {
                    Self::flush_slice(&mut slice, &mut tokens);
                    Self::flush_number(&mut number, &mut tokens, &mut tracking_float);

                    tokens.push(Token::Uknown(c))
                }
            }
        }

        Self::flush_slice(&mut slice, &mut tokens);
        Self::flush_number(&mut number, &mut tokens, &mut tracking_float);

        tokens
    }

    /// Returns the detected alignment of the template
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }
}

#[cfg(test)]
mod tokenization {
    use smacro::s;

    use super::*;

    #[test]
    fn basic() {
        let content = "hello:world";
        let tokens = Template::<'{', '}'>::tokenize(content);

        assert_eq!(
            tokens,
            vec![
                Token::Slice(s!("hello")),
                Token::Symbol(':'),
                Token::Slice(s!("world"))
            ]
        )
    }

    #[test]
    fn with_numbers() {
        let content = "hello:234__world";
        let tokens = Template::<'{', '}'>::tokenize(content);

        assert_eq!(
            tokens,
            vec![
                Token::Slice(s!("hello")),
                Token::Symbol(':'),
                Token::Int(234),
                Token::Slice(s!("__world"))
            ]
        )
    }

    #[test]
    fn with_floats() {
        let content = "-3.40pi:3.14159__e:2.71828";
        let tokens = Template::<'{', '}'>::tokenize(content);

        assert_eq!(
            tokens,
            vec![
                Token::Float(-3.4),
                Token::Slice(s!("pi")),
                Token::Symbol(':'),
                Token::Float(3.14159),
                Token::Slice(s!("__e")),
                Token::Symbol(':'),
                Token::Float(2.71828)
            ]
        );

        let content = "density:.3--Hello!";
        let tokens = Template::<'{', '}'>::tokenize(content);

        assert_eq!(
            tokens,
            vec![
                Token::Slice(s!("density")),
                Token::Symbol(':'),
                Token::Float(0.3),
                Token::Symbol('-'),
                Token::Symbol('-'),
                Token::Slice(s!("Hello")),
                Token::Symbol('!')
            ]
        );
    }

    #[test]
    fn with_negatives() {
        let content = "negative:-42 pos-2134.567itive:42, symbol:-";
        let tokens = Template::<'{', '}'>::tokenize(content);

        assert_eq!(
            tokens,
            vec![
                Token::Slice(s!("negative")),
                Token::Symbol(':'),
                Token::Int(-42),
                Token::Slice(s!(" pos")),
                Token::Float(-2134.567),
                Token::Slice(s!("itive")),
                Token::Symbol(':'),
                Token::Int(42),
                Token::Symbol(','),
                Token::Slice(s!(" symbol")),
                Token::Symbol(':'),
                Token::Symbol('-')
            ]
        );
    }

    #[test]
    fn complex() {
        let content = "hi_world:13.:hi mom!-129-836;#$%-^&-45.6*()";
        let tokens = Template::<'{', '}'>::tokenize(content);

        assert_eq!(
            tokens,
            vec![
                Token::Slice(s!("hi_world")),
                Token::Symbol(':'),
                Token::Float(13.0),
                Token::Symbol(':'),
                Token::Slice(s!("hi mom")),
                Token::Symbol('!'),
                Token::Int(-129),
                Token::Int(-836),
                Token::Symbol(';'),
                Token::Symbol('#'),
                Token::Symbol('$'),
                Token::Symbol('%'),
                Token::Symbol('-'),
                Token::Symbol('^'),
                Token::Symbol('&'),
                Token::Float(-45.6),
                Token::Symbol('*'),
                Token::Symbol('('),
                Token::Symbol(')'),
            ]
        )
    }
}
#[cfg(test)]
mod parsing {
    use super::*;
    use smacro::s;
    use std::fmt::Debug;

    // Mock directive for testing
    #[derive(Debug, Clone)]
    struct MockDirective {
        content: String,
    }

    impl Directive for MockDirective {
        fn execute(&self, _ctx: &Context) -> Result<String, TemplateError> {
            Ok(format!("DIRECTIVE[{}]", self.content))
        }
    }

    // Mock parser that creates simple directives
    struct MockParser;

    impl Parser for MockParser {
        fn parse(_tokens: &[Token], content: &str) -> Option<Box<dyn Directive>> {
            Some(Box::new(MockDirective {
                content: s!(content),
            }))
        }
    }

    // Test helper to create a context
    fn create_test_context() -> Context {
        let mut ctx = HashMap::new();
        ctx.insert("name", Value::String(s!("John")));
        ctx.insert("age", Value::Int(30));
        ctx.insert("active", Value::Bool(true));
        ctx
    }

    #[test]
    fn test_parse_empty_template() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_parse_literal_only() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("Hello World").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_parse_single_directive() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{name}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]");
    }

    #[test]
    fn test_parse_literal_with_directive() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("Hello {name}!").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello DIRECTIVE[name]!");
    }

    #[test]
    fn test_parse_multiple_directives() {
        let template =
            Template::<'{', '}'>::with_parser::<MockParser>("{name} is {age} years old").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name] is DIRECTIVE[age] years old");
    }

    #[test]
    fn test_parse_consecutive_directives() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{name}{age}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]DIRECTIVE[age]");
    }

    #[test]
    fn test_parse_escaped_opening_delimiter() {
        let template =
            Template::<'{', '}'>::with_parser::<MockParser>("{{not a directive}}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "{not a directive}");
    }

    #[test]
    fn test_parse_escaped_closing_delimiter() {
        let template =
            Template::<'{', '}'>::with_parser::<MockParser>("This is }} not escaped").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "This is } not escaped");
    }

    #[test]
    fn test_parse_mixed_escaped_and_directive() {
        let template =
            Template::<'{', '}'>::with_parser::<MockParser>("{{escaped}} {name} }}also escaped{{")
                .unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "{escaped} DIRECTIVE[name] }also escaped{");
    }

    #[test]
    fn test_parse_directive_with_alignment_left() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{name<}").unwrap();
        assert_eq!(template.alignment(), Alignment::Left);
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]");
    }

    #[test]
    fn test_parse_directive_with_alignment_right() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{name>}").unwrap();
        assert_eq!(template.alignment(), Alignment::Right);
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]");
    }

    #[test]
    fn test_parse_directive_with_alignment_center() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{name^}").unwrap();
        assert_eq!(template.alignment(), Alignment::Center);
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name]");
    }

    #[test]
    fn test_parse_default_alignment() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{name}").unwrap();
        assert_eq!(template.alignment(), Alignment::Left);
    }

    #[test]
    fn test_parse_complex_directive_content() {
        let template =
            Template::<'{', '}'>::with_parser::<MockParser>("{user:format:pad:20}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[user:format:pad:20]");
    }

    #[test]
    fn test_parse_directive_with_numbers() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{value:123:pad}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[value:123:pad]");
    }

    #[test]
    fn test_parse_custom_delimiters() {
        let template = Template::<'[', ']'>::with_parser::<MockParser>("Hello [name]!").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello DIRECTIVE[name]!");
    }

    #[test]
    fn test_parse_custom_delimiters_with_escaping() {
        let template =
            Template::<'[', ']'>::with_parser::<MockParser>("[[escaped]] [name] ]]also]]").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "[escaped] DIRECTIVE[name] ]also]");
    }

    #[test]
    fn test_parse_same_delimiters() {
        let template =
            Template::<'|', '|'>::with_parser::<MockParser>("Hello |name| World").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello DIRECTIVE[name] World");
    }

    // Error cases
    #[test]
    fn test_parse_missing_closing_delimiter() {
        let result = Template::<'{', '}'>::with_parser::<MockParser>("Hello {name");
        assert!(matches!(
            result,
            Err(TemplateError::MissingClosedDelimiter('}'))
        ));
    }

    #[test]
    fn test_parse_missing_opening_delimiter() {
        let result = Template::<'{', '}'>::with_parser::<MockParser>("Hello name}");
        assert!(matches!(
            result,
            Err(TemplateError::MissingOpenDelimiter('{'))
        ));
    }

    #[test]
    fn test_parse_nested_delimiters() {
        let result = Template::<'{', '}'>::with_parser::<MockParser>("Hello {name {age}}");
        assert!(matches!(
            result,
            Err(TemplateError::MissingClosedDelimiter('}'))
        ));
    }

    #[test]
    fn test_parse_unbalanced_delimiters_complex() {
        let result = Template::<'{', '}'>::with_parser::<MockParser>("{name} {age");
        assert!(matches!(
            result,
            Err(TemplateError::MissingClosedDelimiter('}'))
        ));
    }

    #[test]
    fn test_parse_extra_closing_delimiter() {
        let result = Template::<'{', '}'>::with_parser::<MockParser>("Hello {name}} extra");

        assert!(matches!(
            result,
            Err(TemplateError::MissingClosedDelimiter('}'))
        ));
    }

    // Parser that fails to parse certain tokens
    struct FailingParser;

    impl Parser for FailingParser {
        fn parse(_tokens: &[Token], content: &str) -> Option<Box<dyn Directive>> {
            if content.contains("fail") {
                None
            } else {
                Some(Box::new(MockDirective {
                    content: content.to_owned(),
                }))
            }
        }
    }

    #[test]
    fn test_parse_directive_parsing_failure() {
        let result = Template::<'{', '}'>::with_parser::<FailingParser>("Hello {fail}");
        assert!(matches!(result, Err(TemplateError::DirectiveParsing(_))));
    }

    #[test]
    fn test_parse_whitespace_in_directive() {
        let template =
            Template::<'{', '}'>::with_parser::<MockParser>("{  name with spaces  }").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[  name with spaces  ]");
    }

    #[test]
    fn test_parse_empty_directive() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[]");
    }

    #[test]
    fn test_parse_directive_with_special_chars() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{name@#$%^&*()}").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name@#$%^&*()]");
    }

    #[test]
    fn test_parse_multiple_alignment_characters() {
        // Only the last character should be treated as alignment
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{name<>^}").unwrap();
        assert_eq!(template.alignment(), Alignment::Center);
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[name<>]");
    }

    #[test]
    fn test_parse_alignment_in_middle_not_treated_as_alignment() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("{na<me}").unwrap();
        assert_eq!(template.alignment(), Alignment::Left); // default
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "DIRECTIVE[na<me]");
    }

    #[test]
    fn test_parse_unicode_content() {
        let template = Template::<'{', '}'>::with_parser::<MockParser>("Hello {名前} 🌟").unwrap();
        let result = template.format(&create_test_context()).unwrap();
        assert_eq!(result, "Hello DIRECTIVE[名前] 🌟");
    }

    #[test]
    fn test_parse_long_template() {
        let long_template = "Start ".repeat(100) + "{name}" + &" End".repeat(100);
        let template = Template::<'{', '}'>::with_parser::<MockParser>(&long_template).unwrap();
        let result = template.format(&create_test_context()).unwrap();
        let expected = "Start ".repeat(100) + "DIRECTIVE[name]" + &" End".repeat(100);
        assert_eq!(result, expected);
    }
}
