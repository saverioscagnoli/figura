use std::hint::black_box;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use figura::Context;
use figura::Template;
use figura::Value;

type CBTemplate = Template<'{', '}'>;

fn simple_string_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_strings");

    // Benchmark: Simple variable replacement
    group.bench_function("single_variable", |b| {
        let template = CBTemplate::compile("Hello, {name}!").unwrap();
        let mut ctx = Context::default();
        ctx.insert("name", Value::static_str("World"));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Multiple variables
    group.bench_function("multiple_variables", |b| {
        let template =
            CBTemplate::compile("Hello, {name}! You are {age} years old and live in {city}.")
                .unwrap();
        let mut ctx = Context::default();
        ctx.insert("name", Value::static_str("Alice"));
        ctx.insert("age", Value::Int(30));
        ctx.insert("city", Value::static_str("New York"));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: No variables (literal only)
    group.bench_function("literal_only", |b| {
        let template = CBTemplate::compile("This is a plain string with no variables.").unwrap();
        let ctx = Context::default();

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Escaped delimiters
    group.bench_function("escaped_delimiters", |b| {
        let template = CBTemplate::compile("Use {{curly braces}} like this: {name}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("name", Value::static_str("example"));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    group.finish();
}

fn complex_pattern_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_patterns");

    // Benchmark: Simple repeat pattern
    group.bench_function("simple_repeat", |b| {
        let template = CBTemplate::compile("{pattern:count}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("pattern", Value::static_str("ABC"));
        ctx.insert("count", Value::Int(10));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Large repeat pattern
    group.bench_function("large_repeat", |b| {
        let template = CBTemplate::compile("{pattern:count}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("pattern", Value::static_str("ABCDEFGHIJ"));
        ctx.insert("count", Value::Int(1000));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Mixed literals and variables
    group.bench_function("mixed_complex", |b| {
        let template =
            CBTemplate::compile("User: {name} | Status: {status} | Repeated: {char:times} | End")
                .unwrap();
        let mut ctx = Context::default();
        ctx.insert("name", Value::static_str("BobTheBuilder"));
        ctx.insert("status", Value::static_str("Active"));
        ctx.insert("char", Value::static_str("*"));
        ctx.insert("times", Value::Int(20));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Many variables
    group.bench_function("many_variables", |b| {
        let template =
            CBTemplate::compile("{v1} {v2} {v3} {v4} {v5} {v6} {v7} {v8} {v9} {v10}").unwrap();
        let mut ctx = Context::default();
        for i in 1..=10 {
            ctx.insert(
                Box::leak(format!("v{}", i).into_boxed_str()),
                Value::Int(i as i64),
            );
        }

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    group.finish();
}

fn very_long_template_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("very_long_templates");

    // Benchmark: Long template with many substitutions
    group.bench_function("long_template", |b| {
        let template_str = (0..100)
            .map(|i| format!("Item {}: {{val{}}}", i, i))
            .collect::<Vec<_>>()
            .join(" | ");

        let template = CBTemplate::compile(&template_str).unwrap();
        let mut ctx = Context::default();
        for i in 0..100 {
            ctx.insert(
                Box::leak(format!("val{}", i).into_boxed_str()),
                Value::Int(i as i64),
            );
        }

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Very long literal string
    group.bench_function("long_literal", |b| {
        let long_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
        let template = CBTemplate::compile(&long_text).unwrap();
        let ctx = Context::default();

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Complex nested patterns
    group.bench_function("complex_nested", |b| {
        let template =
            CBTemplate::compile("Header: {title} | Body: {content:repeat} | Footer: {footer}")
                .unwrap();
        let mut ctx = Context::default();
        ctx.insert("title", Value::static_str("Important Document"));
        ctx.insert("content", Value::static_str("Section "));
        ctx.insert("repeat", Value::Int(50));
        ctx.insert("footer", Value::static_str("End of document"));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    group.finish();
}

fn compilation_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation");

    // Benchmark: Simple template compilation
    group.bench_function("compile_simple", |b| {
        b.iter(|| black_box(CBTemplate::compile("Hello, {name}!").unwrap()));
    });

    // Benchmark: Complex template compilation
    group.bench_function("compile_complex", |b| {
        let template_str = "{a} {b} {c:d} literal text {e} {{escaped}} {f:10}";
        b.iter(|| black_box(CBTemplate::compile(template_str).unwrap()));
    });

    // Benchmark: Long template compilation
    group.bench_function("compile_long", |b| {
        let template_str = (0..50)
            .map(|i| format!("{{var{}}}", i))
            .collect::<Vec<_>>()
            .join(" text ");

        b.iter(|| black_box(CBTemplate::compile(&template_str).unwrap()));
    });

    group.finish();
}

fn realistic_use_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_use_cases");

    // Benchmark: Email template
    group.bench_function("email_template", |b| {
        let template = CBTemplate::compile(
            "Dear {name},\n\nThank you for your order #{order_id}.\n\n\
             Your {item_count} items will be shipped to {address}.\n\n\
             Total: ${total}\n\nBest regards,\nThe Team",
        )
        .unwrap();
        let mut ctx = Context::default();
        ctx.insert("name", Value::static_str("John Doe"));
        ctx.insert("order_id", Value::Int(123456));
        ctx.insert("item_count", Value::Int(3));
        ctx.insert("address", Value::static_str("123 Main St, Anytown, USA"));
        ctx.insert("total", Value::Float(99.99));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: HTML-like template
    group.bench_function("html_template", |b| {
        let template = CBTemplate::compile(
            "<div class=\"user\"><h1>{username}</h1><p>Email: {email}</p>\
             <p>Member since: {year}</p><p>{bio}</p></div>",
        )
        .unwrap();
        let mut ctx = Context::default();
        ctx.insert("username", Value::static_str("alice_wonder"));
        ctx.insert("email", Value::static_str("alice@example.com"));
        ctx.insert("year", Value::Int(2020));
        ctx.insert(
            "bio",
            Value::static_str("Software developer passionate about Rust"),
        );

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Log message template
    group.bench_function("log_template", |b| {
        let template =
            CBTemplate::compile("[{level}] {timestamp} - {module}: {message} (user={user_id})")
                .unwrap();
        let mut ctx = Context::default();
        ctx.insert("level", Value::static_str("INFO"));
        ctx.insert("timestamp", Value::static_str("2024-01-15T10:30:00Z"));
        ctx.insert("module", Value::static_str("auth"));
        ctx.insert("message", Value::static_str("User logged in successfully"));
        ctx.insert("user_id", Value::Int(42));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    group.finish();
}

fn conditional_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("conditionals");

    // Benchmark: Simple boolean conditional
    group.bench_function("simple_boolean", |b| {
        let template = CBTemplate::compile("{flag ? 'yes' : 'no'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("flag", Value::Bool(true));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: String equality comparison
    group.bench_function("string_equality", |b| {
        let template = CBTemplate::compile("{status == 'online' ? 'Active' : 'Inactive'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("status", Value::static_str("online"));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: String inequality comparison
    group.bench_function("string_inequality", |b| {
        let template =
            CBTemplate::compile("{status != 'offline' ? 'Connected' : 'Disconnected'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("status", Value::static_str("online"));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Numeric greater than
    group.bench_function("numeric_greater_than", |b| {
        let template = CBTemplate::compile("{age > 18 ? 'Adult' : 'Minor'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("age", Value::Int(25));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Numeric less than
    group.bench_function("numeric_less_than", |b| {
        let template = CBTemplate::compile("{score < 50 ? 'Fail' : 'Pass'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("score", Value::Int(75));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Greater than or equals
    group.bench_function("greater_than_equals", |b| {
        let template = CBTemplate::compile("{score >= 90 ? 'A' : 'B'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("score", Value::Int(95));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Less than or equals
    group.bench_function("less_than_equals", |b| {
        let template = CBTemplate::compile("{temp <= 32 ? 'Freezing' : 'Above freezing'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("temp", Value::Int(30));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Float comparison
    group.bench_function("float_comparison", |b| {
        let template = CBTemplate::compile("{price > 99.99 ? 'Expensive' : 'Affordable'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("price", Value::Float(120.50));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: NOT operator
    group.bench_function("not_operator", |b| {
        let template = CBTemplate::compile("{!flag ? 'Off' : 'On'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("flag", Value::Bool(false));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Comparing two variables
    group.bench_function("two_variables", |b| {
        let template = CBTemplate::compile("{a == b ? 'Same' : 'Different'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("a", Value::Int(42));
        ctx.insert("b", Value::Int(42));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Variable and literal comparison
    group.bench_function("variable_literal", |b| {
        let template =
            CBTemplate::compile("{role == 'admin' ? 'Full Access' : 'Limited Access'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("role", Value::static_str("admin"));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Multiple conditionals in one template
    group.bench_function("multiple_conditionals", |b| {
        let template = CBTemplate::compile(
            "{x > 0 ? 'positive' : 'non-positive'} and {y > 0 ? 'positive' : 'non-positive'}",
        )
        .unwrap();
        let mut ctx = Context::default();
        ctx.insert("x", Value::Int(5));
        ctx.insert("y", Value::Int(-3));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Conditional with variable results
    group.bench_function("conditional_variable_results", |b| {
        let template = CBTemplate::compile("{premium ? gold_msg : silver_msg}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("premium", Value::Bool(true));
        ctx.insert("gold_msg", Value::static_str("Premium User"));
        ctx.insert("silver_msg", Value::static_str("Standard User"));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Complex conditional in realistic template
    group.bench_function("realistic_user_status", |b| {
        let template = CBTemplate::compile(
            "User: {name} | Status: {online ? 'Online ✓' : 'Offline ✗'} | \
             Access: {role == 'admin' ? 'Full' : 'Limited'}",
        )
        .unwrap();
        let mut ctx = Context::default();
        ctx.insert("name", Value::static_str("Alice"));
        ctx.insert("online", Value::Bool(true));
        ctx.insert("role", Value::static_str("admin"));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Conditional with unicode
    group.bench_function("unicode_conditional", |b| {
        let template = CBTemplate::compile("{success ? '✅ Success' : '❌ Failed'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("success", Value::Bool(true));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    // Benchmark: Truthy/falsy integer
    group.bench_function("truthy_integer", |b| {
        let template = CBTemplate::compile("{count ? 'Has items' : 'Empty'}").unwrap();
        let mut ctx = Context::default();
        ctx.insert("count", Value::Int(5));

        b.iter(|| black_box(template.format(&ctx).unwrap()));
    });

    group.finish();
}

fn conditional_compilation_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("conditional_compilation");

    // Benchmark: Compile simple conditional
    group.bench_function("compile_simple", |b| {
        b.iter(|| black_box(CBTemplate::compile("{flag ? 'yes' : 'no'}").unwrap()));
    });

    // Benchmark: Compile comparison conditional
    group.bench_function("compile_comparison", |b| {
        b.iter(|| black_box(CBTemplate::compile("{age > 18 ? 'Adult' : 'Minor'}").unwrap()));
    });

    // Benchmark: Compile complex conditional
    group.bench_function("compile_complex", |b| {
        b.iter(|| {
            black_box(
                CBTemplate::compile(
                    "Status: {x > 0 ? 'positive' : 'negative'} and {y == z ? 'equal' : 'not equal'}",
                )
                .unwrap(),
            )
        });
    });

    // Benchmark: Compile NOT conditional
    group.bench_function("compile_not", |b| {
        b.iter(|| black_box(CBTemplate::compile("{!flag ? 'Off' : 'On'}").unwrap()));
    });

    group.finish();
}

criterion_group!(
    benches,
    simple_string_benchmarks,
    complex_pattern_benchmarks,
    very_long_template_benchmarks,
    compilation_benchmarks,
    realistic_use_cases,
    conditional_benchmarks,
    conditional_compilation_benchmarks
);
criterion_main!(benches);
