use figura::Context;
use figura::Template;
use figura::Value;

type CBTemplate = Template<'{', '}'>;

fn main() {
    let mut ctx = Context::default();
    let template = CBTemplate::compile("Hello, my name is {name}!").unwrap();

    ctx.insert("name", Value::static_str("John"));
    ctx.insert("count", Value::Int(4));

    let output = template.format(&ctx).unwrap();

    println!("{}", output);

    let template =
        CBTemplate::compile("This will be repeated {count} times {'Abbacchio':count}").unwrap();

    let output = template.format(&ctx).unwrap();

    println!("{}", output);

    let template =
        CBTemplate::compile("Status: {status == 'offline' ? 'Offline :(' : 'Online :)'}").unwrap();

    ctx.insert("status", Value::static_str("offline"));

    let output = template.format(&ctx).unwrap();

    println!("{}", output);
}
