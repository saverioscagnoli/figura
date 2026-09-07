use figura::Context;
use figura::Template;
use figura::Value;

fn main() {
    let mut ctx = Context::default();
    ctx.insert("char", Value::static_str("="));
    ctx.insert("count", Value::Int(50));
    ctx.insert("title", Value::static_str("HEADER"));

    let template = Template::<'{', '}'>::compile("{char:count}\n{title}\n{char:count}").unwrap();

    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("pattern", Value::static_str("* "));
    ctx.insert("times", Value::Int(10));

    let template = Template::<'{', '}'>::compile("Pattern: {pattern:times}").unwrap();

    println!("{}", template.format(&ctx).unwrap());

    let template = Template::<'{', '}'>::compile("{'#':5} Progress Bar {'#':5}").unwrap();

    println!("{}", template.format(&Context::default()).unwrap());
}
