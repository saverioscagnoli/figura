use figura::Context;
use figura::Template;
use figura::Value;

fn main() {
    let mut ctx = Context::default();

    ctx.insert("title", Value::static_str("User Statistics"));
    ctx.insert("col1_width", Value::Int(20));
    ctx.insert("col2_width", Value::Int(15));
    ctx.insert("col3_width", Value::Int(10));

    ctx.insert("name1", Value::static_str("Alice"));
    ctx.insert("score1", Value::Int(95));
    ctx.insert("active1", Value::Bool(true));

    ctx.insert("name2", Value::static_str("Bob"));
    ctx.insert("score2", Value::Int(87));
    ctx.insert("active2", Value::Bool(false));

    ctx.insert("name3", Value::static_str("Charlie"));
    ctx.insert("score3", Value::Int(92));
    ctx.insert("active3", Value::Bool(true));

    let template = Template::<'{', '}'>::compile(
        "\
{title}
{'-':60}
| {'Name'}{' ':14} | {'Score'}{' ':9} | Status   |
{'-':60}
| {name1}{' ':14} | {score1}{' ':9} | {active1 ? '✓ Active' : '✗ Inactive'} |
| {name2}{' ':16} | {score2}{' ':9} | {active2 ? '✓ Active' : '✗ Inactive'} |
| {name3}{' ':12} | {score3}{' ':9} | {active3 ? '✓ Active' : '✗ Inactive'} |
{'-':60}
Total: {!active2 ? '1' : '0'} inactive users
",
    )
    .unwrap();

    let output = template.format(&ctx).unwrap();
    println!("{}", output);
}
