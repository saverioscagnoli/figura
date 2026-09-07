use std::borrow::Cow;

use figura::Argument;
use figura::Context;
use figura::Directive;
use figura::EmptyDirective;
use figura::Parser;
use figura::ReplaceDirective;
use figura::Template;
use figura::Token;
use figura::Value;

struct MathParser;

impl Parser for MathParser {
    fn parse(tokens: &[Token]) -> Option<Box<dyn Directive + Send + Sync>> {
        match tokens {
            [Token::Ident(var)] => Some(Box::new(ReplaceDirective(Argument::variable(
                Cow::Owned(var.to_string()),
            )))),

            [Token::Ident(left), Token::Plus, Token::Ident(right)] => {
                Some(Box::new(AddDirective {
                    left: Cow::Owned(left.to_string()),
                    right: Cow::Owned(right.to_string()),
                }))
            }

            [Token::Ident(left), Token::Minus, Token::Ident(right)] => {
                Some(Box::new(SubtractDirective {
                    left: Cow::Owned(left.to_string()),
                    right: Cow::Owned(right.to_string()),
                }))
            }

            [Token::Ident(left), Token::Star, Token::Ident(right)] => {
                Some(Box::new(MultiplyDirective {
                    left: Cow::Owned(left.to_string()),
                    right: Cow::Owned(right.to_string()),
                }))
            }

            [Token::Ident(left), Token::Slash, Token::Ident(right)] => {
                Some(Box::new(DivideDirective {
                    left: Cow::Owned(left.to_string()),
                    right: Cow::Owned(right.to_string()),
                }))
            }

            [Token::Ident(var), Token::Star, Token::Int(num)] => {
                Some(Box::new(MultiplyByLiteralDirective {
                    var: Cow::Owned(var.to_string()),
                    multiplier: num.parse().unwrap_or(1),
                }))
            }

            [Token::Ident(var), Token::Plus, Token::Int(num)] => {
                Some(Box::new(AddLiteralDirective {
                    var: Cow::Owned(var.to_string()),
                    addend: num.parse().unwrap_or(0),
                }))
            }

            _ => Some(Box::new(EmptyDirective)),
        }
    }
}

struct AddDirective {
    left: Cow<'static, str>,
    right: Cow<'static, str>,
}

impl Directive for AddDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, figura::DirectiveError> {
        let left_val = ctx
            .get(self.left.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(0);

        let right_val = ctx
            .get(self.right.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(0);

        Ok(Cow::Owned((left_val + right_val).to_string()))
    }
}

struct SubtractDirective {
    left: Cow<'static, str>,
    right: Cow<'static, str>,
}

impl Directive for SubtractDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, figura::DirectiveError> {
        let left_val = ctx
            .get(self.left.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(0);

        let right_val = ctx
            .get(self.right.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(0);

        Ok(Cow::Owned((left_val - right_val).to_string()))
    }
}

struct MultiplyDirective {
    left: Cow<'static, str>,
    right: Cow<'static, str>,
}

impl Directive for MultiplyDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, figura::DirectiveError> {
        let left_val = ctx
            .get(self.left.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(0);

        let right_val = ctx
            .get(self.right.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(0);

        Ok(Cow::Owned((left_val * right_val).to_string()))
    }
}

struct DivideDirective {
    left: Cow<'static, str>,
    right: Cow<'static, str>,
}

impl Directive for DivideDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, figura::DirectiveError> {
        let left_val = ctx
            .get(self.left.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(0);

        let right_val = ctx
            .get(self.right.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(1);

        Ok(Cow::Owned((left_val / right_val).to_string()))
    }
}

struct MultiplyByLiteralDirective {
    var: Cow<'static, str>,
    multiplier: i64,
}

impl Directive for MultiplyByLiteralDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, figura::DirectiveError> {
        let val = ctx
            .get(self.var.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(0);

        Ok(Cow::Owned((val * self.multiplier).to_string()))
    }
}

struct AddLiteralDirective {
    var: Cow<'static, str>,
    addend: i64,
}

impl Directive for AddLiteralDirective {
    fn exec(&self, ctx: &Context) -> Result<Cow<'static, str>, figura::DirectiveError> {
        let val = ctx
            .get(self.var.as_ref())
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                Value::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(0);

        Ok(Cow::Owned((val + self.addend).to_string()))
    }
}

fn main() {
    let mut ctx = Context::default();
    ctx.insert("x", Value::Int(10));
    ctx.insert("y", Value::Int(5));
    ctx.insert("a", Value::Int(100));
    ctx.insert("b", Value::Int(25));

    let template =
        Template::<'{', '}'>::compile_with_parser::<MathParser>("x = {x}, y = {y}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template =
        Template::<'{', '}'>::compile_with_parser::<MathParser>("x + y = {x + y}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template =
        Template::<'{', '}'>::compile_with_parser::<MathParser>("x - y = {x - y}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template =
        Template::<'{', '}'>::compile_with_parser::<MathParser>("x * y = {x * y}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template =
        Template::<'{', '}'>::compile_with_parser::<MathParser>("a / b = {a / b}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template =
        Template::<'{', '}'>::compile_with_parser::<MathParser>("x * 3 = {x * 3}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template =
        Template::<'{', '}'>::compile_with_parser::<MathParser>("y + 10 = {y + 10}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template = Template::<'{', '}'>::compile_with_parser::<MathParser>(
        "Result: {x + y} + {a - b} = {x * 2}",
    )
    .unwrap();
    println!("{}", template.format(&ctx).unwrap());
}
