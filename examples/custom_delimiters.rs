use figura::Context;
use figura::Template;
use figura::Value;

fn main() {
    let mut ctx = Context::default();
    ctx.insert("title", Value::static_str("Custom Delimiters"));
    ctx.insert("count", Value::Int(5));

    let template = Template::<'<', '>'>::compile("Title: <title> | Stars: <'★':count>").unwrap();

    let output = template.format(&ctx).unwrap();
    println!("{}", output);

    let template =
        Template::<'[', ']'>::compile("Using brackets: [title] with [count] items").unwrap();

    let output = template.format(&ctx).unwrap();
    println!("{}", output);

    let template = Template::<'%', '%'>::compile("Percent signs: %title% %%escaped%%").unwrap();

    let output = template.format(&ctx).unwrap();
    println!("{}", output);
}
