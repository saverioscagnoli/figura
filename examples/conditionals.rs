use figura::Context;
use figura::Template;
use figura::Value;

fn main() {
    let mut ctx = Context::default();

    ctx.insert("is_admin", Value::Bool(true));
    ctx.insert("score", Value::Int(85));
    ctx.insert("status", Value::static_str("active"));
    ctx.insert("temperature", Value::Float(22.5));

    let template =
        Template::<'{', '}'>::compile("Access: {is_admin ? 'GRANTED' : 'DENIED'}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template = Template::<'{', '}'>::compile("Grade: {score >= 90 ? 'A' : 'not-A'}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("score", Value::Int(88));
    let template = Template::<'{', '}'>::compile("Grade: {score >= 80 ? 'B' : 'below-B'}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("score", Value::Int(72));
    let template = Template::<'{', '}'>::compile("Grade: {score < 80 ? 'C' : 'above-C'}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("score", Value::Int(85));

    let template =
        Template::<'{', '}'>::compile("Status: {status == 'active' ? 'Online' : 'Offline'}")
            .unwrap();
    println!("{}", template.format(&ctx).unwrap());

    let template =
        Template::<'{', '}'>::compile("Temp: {temperature > 20.0 ? 'Warm' : 'Cold'}").unwrap();
    println!("{}", template.format(&ctx).unwrap());

    ctx.insert("is_admin", Value::Bool(false));
    let template =
        Template::<'{', '}'>::compile("Access: {!is_admin ? 'DENIED' : 'GRANTED'}").unwrap();
    println!("{}", template.format(&ctx).unwrap());
}
