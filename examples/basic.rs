use figura::Context;
use figura::Template;
use figura::Value;

fn main() {
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("Alice"));
    ctx.insert("age", Value::Int(30));
    ctx.insert("city", Value::static_str("Boston"));

    let template =
        Template::<'{', '}'>::compile("Hello {name}! You are {age} years old and live in {city}.")
            .unwrap();

    let output = template.format(&ctx).unwrap();
    println!("{}", output);
}
