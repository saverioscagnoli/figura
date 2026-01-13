use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::{collections::HashMap, hint::black_box};

// Adjust this import to match your crate name
use figura::{Context, DefaultParser, Template, Token, Value};

fn bench_tokenizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");

    let inputs = [
        ("simple_ident", "foo"),
        ("number", "12345"),
        ("float", "123.456"),
        ("negative", "-42"),
        ("expression", "foo == bar && baz != 42"),
        ("complex", "items:count + 10 >= limit"),
        ("with_symbols", "hello:world!@#$%"),
    ];

    for (name, input) in inputs {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), input, |b, input| {
            b.iter(|| {
                let tokens = Template::<'{', '}'>::tokenize(black_box(input));
                black_box(tokens)
            });
        });
    }

    group.finish();
}

// Helper to expose tokenize for benchmarking (add this to your lib if needed)
// Or use a wrapper that calls parse and measures just that part
trait TokenizeAccess {
    fn tokenize(input: &str) -> Vec<Token>;
}

impl<const O: char, const C: char> TokenizeAccess for Template<O, C> {
    fn tokenize(input: &str) -> Vec<Token> {
        // You may need to make tokenize public or use a test helper
        Template::<O, C>::tokenize(input)
    }
}

fn bench_template_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    let templates = [
        ("literal_only", "Hello, World!"),
        ("single_var", "Hello, {name}!"),
        (
            "multiple_vars",
            "Hello, {first} {last}! You are {age} years old.",
        ),
        ("escaped_delim", "Use {{braces}} for templates"),
        ("repeat", "{star:10}"),
        ("conditional", "{is_admin?Admin:User}"),
        ("comparison", "{age>=18?Adult:Minor}"),
        (
            "mixed",
            "Dear {name},\n\nYour order #{order_id} is ready.\nTotal: ${total}\n\nThanks!",
        ),
    ];

    for (name, template) in templates {
        group.throughput(Throughput::Bytes(template.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), template, |b, template| {
            b.iter(|| Template::<'{', '}'>::parse(black_box(template)).unwrap());
        });
    }

    // Large template
    let large_template = "{greeting} {name}! ".repeat(100);
    group.throughput(Throughput::Bytes(large_template.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "large_100_vars"),
        &large_template,
        |b, template| {
            b.iter(|| Template::<'{', '}'>::parse(black_box(template)).unwrap());
        },
    );

    group.finish();
}

fn bench_template_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("format");

    // Single variable
    {
        let template = Template::<'{', '}'>::parse("Hello, {name}!").unwrap();
        let mut ctx: Context = HashMap::new();
        ctx.insert("name", Value::String("World".to_string()));

        group.bench_function("single_var", |b| {
            b.iter(|| {
                let result = template.format(black_box(&ctx)).unwrap();
                black_box(result)
            });
        });
    }

    // Multiple variables
    {
        let template = Template::<'{', '}'>::parse(
            "Hello, {first} {last}! You are {age} years old and have ${balance}.",
        )
        .unwrap();

        let mut ctx: Context = HashMap::new();
        ctx.insert("first", Value::String("John".to_string()));
        ctx.insert("last", Value::String("Doe".to_string()));
        ctx.insert("age", Value::Int(30));
        ctx.insert("balance", Value::Float(1234.56));

        group.bench_function("multiple_vars", |b| {
            b.iter(|| {
                let result = template.format(black_box(&ctx)).unwrap();
                black_box(result)
            });
        });
    }

    // Repeat directive
    {
        let template = Template::<'{', '}'>::parse("{pattern:count}").unwrap();

        let mut ctx: Context = HashMap::new();
        ctx.insert("pattern", Value::String("*".to_string()));
        ctx.insert("count", Value::Int(50));

        group.bench_function("repeat_directive", |b| {
            b.iter(|| {
                let result = template.format(black_box(&ctx)).unwrap();
                black_box(result)
            });
        });
    }

    // Repeat with literal count
    {
        let template = Template::<'{', '}'>::parse("{pattern:50}").unwrap();

        let mut ctx: Context = HashMap::new();
        ctx.insert("pattern", Value::String("*".to_string()));

        group.bench_function("repeat_literal_count", |b| {
            b.iter(|| {
                let result = template.format(black_box(&ctx)).unwrap();
                black_box(result)
            });
        });
    }

    // Literal only (no substitution)
    {
        let template =
            Template::<'{', '}'>::parse("This is a static string with no variables at all.")
                .unwrap();
        let ctx: Context = HashMap::new();

        group.bench_function("literal_only", |b| {
            b.iter(|| {
                let result = template.format(black_box(&ctx)).unwrap();
                black_box(result)
            });
        });
    }

    // Conditional directive (simple boolean)
    {
        let template = Template::<'{', '}'>::parse("{is_admin?Admin Panel:User Panel}").unwrap();

        let mut ctx: Context = HashMap::new();
        ctx.insert("is_admin", Value::Bool(true));

        group.bench_function("conditional_bool", |b| {
            b.iter(|| {
                let result = template.format(black_box(&ctx)).unwrap();
                black_box(result)
            });
        });
    }

    // Conditional directive (comparison)
    {
        let template = Template::<'{', '}'>::parse("{age>=18?Adult:Minor}").unwrap();

        let mut ctx: Context = HashMap::new();
        ctx.insert("age", Value::Int(25));

        group.bench_function("conditional_comparison", |b| {
            b.iter(|| {
                let result = template.format(black_box(&ctx)).unwrap();
                black_box(result)
            });
        });
    }

    // Switch directive
    {
        let template = Template::<'{', '}'>::parse(
            "{[status](active:Online)(inactive:Offline)(maintenance:Under Maintenance)}",
        )
        .unwrap();

        let mut ctx: Context = HashMap::new();
        ctx.insert("status", Value::String("active".to_string()));

        group.bench_function("switch_directive", |b| {
            b.iter(|| {
                let result = template.format(black_box(&ctx)).unwrap();
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_e2e(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    // Realistic email template
    let email_template = r#"Dear {name},

Thank you for your order #{order_id}!

Items ordered: {item_count}
Subtotal: ${subtotal}
Tax: ${tax}
Total: ${total}

Your order will be shipped to:
{address}

Best regards,
The Team"#;

    let mut ctx: Context = HashMap::new();
    ctx.insert("name", Value::String("Alice Smith".to_string()));
    ctx.insert("order_id", Value::Int(12345));
    ctx.insert("item_count", Value::Int(3));
    ctx.insert("subtotal", Value::Float(99.99));
    ctx.insert("tax", Value::Float(8.50));
    ctx.insert("total", Value::Float(108.49));
    ctx.insert(
        "address",
        Value::String("123 Main St, City, ST 12345".to_string()),
    );

    group.bench_function("email_parse_and_format", |b| {
        b.iter(|| {
            let template = Template::<'{', '}'>::parse(black_box(email_template)).unwrap();
            let result = template.format(&ctx).unwrap();
            black_box(result)
        });
    });

    let template = Template::<'{', '}'>::parse(email_template).unwrap();

    group.bench_function("email_format_only", |b| {
        b.iter(|| {
            let result = template.format(black_box(&ctx)).unwrap();
            black_box(result)
        });
    });

    // Template with conditionals
    let conditional_template = r#"Welcome {name}!
Status: {is_premium?Premium Member:Free User}
Access Level: {role==admin?Full Access:Limited Access}
Age Group: {age>=18?Adult:Minor}"#;

    let mut ctx2: Context = HashMap::new();
    ctx2.insert("name", Value::String("Bob".to_string()));
    ctx2.insert("is_premium", Value::Bool(true));
    ctx2.insert("role", Value::String("admin".to_string()));
    ctx2.insert("age", Value::Int(25));

    let conditional_tpl = Template::<'{', '}'>::parse(conditional_template).unwrap();

    group.bench_function("conditional_template", |b| {
        b.iter(|| {
            let result = conditional_tpl.format(black_box(&ctx2)).unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn bench_custom_delimiters(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_delimiters");

    let mut ctx: Context = HashMap::new();
    ctx.insert("name", Value::String("World".to_string()));

    group.bench_function("curly_braces", |b| {
        b.iter(|| {
            let t = Template::<'{', '}'>::parse(black_box("Hello, {name}!")).unwrap();
            let result = t.format(&ctx).unwrap();
            black_box(result)
        });
    });

    group.bench_function("angle_brackets", |b| {
        b.iter(|| {
            let t = Template::<'<', '>'>::parse(black_box("Hello, <name>!")).unwrap();
            let result = t.format(&ctx).unwrap();
            black_box(result)
        });
    });

    group.bench_function("percent_signs", |b| {
        b.iter(|| {
            let t = Template::<'%', '%'>::parse(black_box("Hello, %name%!")).unwrap();
            let result = t.format(&ctx).unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn bench_directives(c: &mut Criterion) {
    let mut group = c.benchmark_group("directives");

    // Replace directive variations
    {
        let mut ctx: Context = HashMap::new();
        ctx.insert("short", Value::String("hi".to_string()));
        ctx.insert("medium", Value::String("hello world".to_string()));
        ctx.insert(
            "long",
            Value::String("this is a much longer string value".to_string()),
        );
        ctx.insert("number", Value::Int(42));
        ctx.insert("float", Value::Float(3.14159));
        ctx.insert("static_str", Value::Str("static"));

        let tpl_short = Template::<'{', '}'>::parse("{short}").unwrap();
        let tpl_medium = Template::<'{', '}'>::parse("{medium}").unwrap();
        let tpl_long = Template::<'{', '}'>::parse("{long}").unwrap();
        let tpl_number = Template::<'{', '}'>::parse("{number}").unwrap();
        let tpl_float = Template::<'{', '}'>::parse("{float}").unwrap();
        let tpl_static = Template::<'{', '}'>::parse("{static_str}").unwrap();

        group.bench_function("replace_short_string", |b| {
            b.iter(|| black_box(tpl_short.format(&ctx).unwrap()));
        });

        group.bench_function("replace_medium_string", |b| {
            b.iter(|| black_box(tpl_medium.format(&ctx).unwrap()));
        });

        group.bench_function("replace_long_string", |b| {
            b.iter(|| black_box(tpl_long.format(&ctx).unwrap()));
        });

        group.bench_function("replace_int", |b| {
            b.iter(|| black_box(tpl_number.format(&ctx).unwrap()));
        });

        group.bench_function("replace_float", |b| {
            b.iter(|| black_box(tpl_float.format(&ctx).unwrap()));
        });

        group.bench_function("replace_static_str", |b| {
            b.iter(|| black_box(tpl_static.format(&ctx).unwrap()));
        });
    }

    // Repeat directive with different counts
    {
        let mut ctx: Context = HashMap::new();
        ctx.insert("char", Value::String("*".to_string()));

        for count in [5, 10, 50, 100, 500] {
            let template_str = format!("{{char:{}}}", count);
            let tpl = Template::<'{', '}'>::parse(&template_str).unwrap();

            group.bench_function(format!("repeat_{}", count), |b| {
                b.iter(|| black_box(tpl.format(&ctx).unwrap()));
            });
        }
    }

    group.finish();
}

fn bench_comparison_operators(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_ops");

    let mut ctx: Context = HashMap::new();
    ctx.insert("a", Value::Int(10));
    ctx.insert("b", Value::Int(20));
    ctx.insert("x", Value::Float(3.14));
    ctx.insert("name", Value::String("alice".to_string()));

    let operators = [
        ("equal", "{a==10?yes:no}"),
        ("not_equal", "{a!=20?yes:no}"),
        ("less_than", "{a<20?yes:no}"),
        ("less_than_eq", "{a<=10?yes:no}"),
        ("greater_than", "{b>10?yes:no}"),
        ("greater_than_eq", "{b>=20?yes:no}"),
        ("string_equal", "{name==alice?yes:no}"),
    ];

    for (name, template_str) in operators {
        let tpl = Template::<'{', '}'>::parse(template_str).unwrap();
        group.bench_function(name, |b| {
            b.iter(|| black_box(tpl.format(&ctx).unwrap()));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_template_parse,
    bench_template_format,
    bench_e2e,
    bench_custom_delimiters,
    bench_directives,
    bench_comparison_operators,
);

criterion_main!(benches);
