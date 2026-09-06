<div align="center">

<h1>figura</h1>

<pre>
.---------------.
| J             |
|   .      //// |
|  / \    |o o| |
| (_,_)   | < | |
|   |     |___| |
|         /   \ |
|        |     ||
'---------------'
</pre>

[![crates.io](https://img.shields.io/crates/v/figura.svg?style=flat-square)](https://crates.io/crates/figura)
[![docs.rs](https://img.shields.io/docsrs/figura?style=flat-square)](https://docs.rs/figura)
[![downloads](https://img.shields.io/crates/d/figura.svg?style=flat-square)](https://crates.io/crates/figura)
[![license](https://img.shields.io/crates/l/figura.svg?style=flat-square)](LICENSE)

<p>A small template engine for Rust. Compile a string once, format it with a context as many times as you want.</p>

<code>figura = "3.1.0"</code>

</div>

## Usage

```rust
use figura::{Context, Template, Value};

let mut ctx = Context::new();
ctx.insert("name", Value::static_str("Alice"));
ctx.insert("count", Value::Int(3));

let template = Template::<'{', '}'>::compile("Hello {name}! Stars: {'*':count}").unwrap();

assert_eq!(template.format(&ctx).unwrap(), "Hello Alice! Stars: ***");
```

Delimiters are const generic params, so pick whatever you like: `Template::<'<', '>'>`, `Template::<'[', ']'>`, or `Template::<'%', '%'>` with the same char on both sides.

## Syntax

| What | Example | Result |
| --- | --- | --- |
| Variable | `{name}` | the value from the context |
| Literal | `{'hi'}` | `hi` |
| Repeat | `{'-':50}` | 50 dashes |
| Ternary | `{active ? 'on' : 'off'}` | one of the two branches |
| Comparison | `{age >= 18 ? 'yes' : 'no'}` | `==`, `!=`, `>`, `<`, `>=`, `<=` |
| Negation | `{!enabled ? 'no' : 'yes'}` | flips the condition |
| Escape | `{{literal}}` | `{literal}` |

Values are `Value::static_str`, `Value::owned_str`, `Value::Int`, `Value::Float` and `Value::Bool`. A `Context` is just a `HashMap<&'static str, Value>`.

## Custom parsers

Implement `Parser` to turn tokens into your own `Directive`, then `Template::<'{', '}'>::compile_with_parser::<MathParser>("{x + y}")`. There's a working one in `examples/custom_parser.rs`.

Other examples: `cargo run --example basic`, `table`, `report`.

## License

MIT
