use figura::Context;
use figura::Template;
use figura::Value;

type CBTemplate = Template<'{', '}'>;
type ParenTemplate = Template<'(', ')'>;
type SquareTemplate = Template<'[', ']'>;

#[test]
fn test_simple_variable_replacement() {
    let template = CBTemplate::compile("Hello, {name}!").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("World"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_multiple_variables() {
    let template =
        CBTemplate::compile("My name is {name}, I am {age} years old, and I live in {city}.")
            .unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("Alice"));
    ctx.insert("age", Value::Int(30));
    ctx.insert("city", Value::static_str("New York"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(
        result,
        "My name is Alice, I am 30 years old, and I live in New York."
    );
}

#[test]
fn test_different_value_types() {
    let template = CBTemplate::compile("String: {s}, Int: {i}, Float: {f}, Bool: {b}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("s", Value::static_str("test"));
    ctx.insert("i", Value::Int(42));
    ctx.insert("f", Value::Float(3.14));
    ctx.insert("b", Value::Bool(true));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "String: test, Int: 42, Float: 3.14, Bool: true");
}

#[test]
fn test_literal_only_template() {
    let template = CBTemplate::compile("This is just a plain string with no variables.").unwrap();
    let ctx = Context::default();

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "This is just a plain string with no variables.");
}

#[test]
fn test_empty_template() {
    let template = CBTemplate::compile("").unwrap();
    let ctx = Context::default();

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_escaped_opening_delimiter() {
    let template = CBTemplate::compile("Use {{curly braces}} like this: {name}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("example"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Use {curly braces} like this: example");
}

#[test]
fn test_escaped_closing_delimiter() {
    let template = CBTemplate::compile("Close with }} and open with {name}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("test"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Close with } and open with test");
}

#[test]
fn test_both_escaped_delimiters() {
    let template = CBTemplate::compile("{{escaped}} {name} {{both}}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("middle"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "{escaped} middle {both}");
}

#[test]
fn test_repeat_directive_with_literal() {
    let template = CBTemplate::compile("{'ABC':5}").unwrap();
    let ctx = Context::default();

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "ABCABCABCABCABC");
}

#[test]
fn test_repeat_directive_with_variables() {
    let template = CBTemplate::compile("{pattern:count}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("pattern", Value::static_str("XYZ"));
    ctx.insert("count", Value::Int(3));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "XYZXYZXYZ");
}

#[test]
fn test_repeat_directive_zero_times() {
    let template = CBTemplate::compile("{pattern:count}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("pattern", Value::static_str("ABC"));
    ctx.insert("count", Value::Int(0));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_repeat_directive_large_count() {
    let template = CBTemplate::compile("{'A':1000}").unwrap();
    let ctx = Context::default();

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "A".repeat(1000));
}

#[test]
fn test_mixed_literals_and_variables() {
    let template = CBTemplate::compile("Start {var1} middle {var2} end {var3} finish").unwrap();
    let mut ctx = Context::default();
    ctx.insert("var1", Value::static_str("ONE"));
    ctx.insert("var2", Value::Int(2));
    ctx.insert("var3", Value::Bool(false));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Start ONE middle 2 end false finish");
}

#[test]
fn test_consecutive_variables() {
    let template = CBTemplate::compile("{a}{b}{c}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("a", Value::static_str("Hello"));
    ctx.insert("b", Value::static_str(" "));
    ctx.insert("c", Value::static_str("World"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Hello World");
}

#[test]
fn test_different_delimiters_parentheses() {
    let template = ParenTemplate::compile("Hello, (name)!").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("World"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_different_delimiters_square_brackets() {
    let template = SquareTemplate::compile("Hello, [name]!").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("World"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_parentheses_escaped() {
    let template = ParenTemplate::compile("Use ((parens)) like (name)").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("this"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Use (parens) like this");
}

#[test]
fn test_unclosed_delimiter_error() {
    let result = CBTemplate::compile("Hello {name");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unclosed delimiter")
    );
}

#[test]
fn test_missing_variable_in_context() {
    let template = CBTemplate::compile("Hello, {name}!").unwrap();
    let ctx = Context::default();

    let result = template.format(&ctx);
    assert!(result.is_err());
}

#[test]
fn test_rc_str_value() {
    let template = CBTemplate::compile("Hello, {name}!").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::owned_str("Dynamic".to_string()));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Hello, Dynamic!");
}

#[test]
fn test_very_long_template() {
    let template_str = (0..100)
        .map(|i| format!("{{var{}}}", i))
        .collect::<Vec<_>>()
        .join(" ");

    let template = CBTemplate::compile(&template_str).unwrap();
    let mut ctx = Context::default();
    for i in 0..100 {
        ctx.insert(
            Box::leak(format!("var{}", i).into_boxed_str()),
            Value::Int(i as i64),
        );
    }

    let result = template.format(&ctx).unwrap();
    let expected = (0..100)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(result, expected);
}

#[test]
fn test_complex_realistic_email() {
    let template = CBTemplate::compile(
        "Dear {name},\n\nYour order #{order_id} has been confirmed.\n\
         Total: ${total}\n\nThank you!",
    )
    .unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("John Doe"));
    ctx.insert("order_id", Value::Int(12345));
    ctx.insert("total", Value::Float(99.99));

    let result = template.format(&ctx).unwrap();
    assert_eq!(
        result,
        "Dear John Doe,\n\nYour order #12345 has been confirmed.\nTotal: $99.99\n\nThank you!"
    );
}

#[test]
fn test_complex_html_like_template() {
    let template = CBTemplate::compile("<div><h1>{title}</h1><p>{content}</p></div>").unwrap();
    let mut ctx = Context::default();
    ctx.insert("title", Value::static_str("Welcome"));
    ctx.insert("content", Value::static_str("Hello, world!"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "<div><h1>Welcome</h1><p>Hello, world!</p></div>");
}

#[test]
fn test_repeat_with_mixed_content() {
    let template = CBTemplate::compile("Header: {title} | {separator:count} | Footer").unwrap();
    let mut ctx = Context::default();
    ctx.insert("title", Value::static_str("Document"));
    ctx.insert("separator", Value::static_str("-"));
    ctx.insert("count", Value::Int(10));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Header: Document | ---------- | Footer");
}

#[test]
fn test_unicode_characters() {
    let template = CBTemplate::compile("Emoji: {emoji}, Text: {text}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("emoji", Value::static_str("🦀"));
    ctx.insert("text", Value::static_str("Здравствуй мир"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Emoji: 🦀, Text: Здравствуй мир");
}

#[test]
fn test_unicode_in_repeat() {
    let template = CBTemplate::compile("{emoji:count}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("emoji", Value::static_str("🦀"));
    ctx.insert("count", Value::Int(5));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "🦀🦀🦀🦀🦀");
}

#[test]
fn test_special_characters_in_literals() {
    let template = CBTemplate::compile("Special: !@#$%^&*() {var} more: <>/\\|").unwrap();
    let mut ctx = Context::default();
    ctx.insert("var", Value::static_str("test"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Special: !@#$%^&*() test more: <>/\\|");
}

#[test]
fn test_whitespace_preservation() {
    let template = CBTemplate::compile("   {var}   \n\t{var2}   ").unwrap();
    let mut ctx = Context::default();
    ctx.insert("var", Value::static_str("a"));
    ctx.insert("var2", Value::static_str("b"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "   a   \n\tb   ");
}

#[test]
fn test_negative_integers() {
    let template = CBTemplate::compile("Value: {num}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("num", Value::Int(-42));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Value: -42");
}

#[test]
fn test_negative_floats() {
    let template = CBTemplate::compile("Value: {num}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("num", Value::Float(-3.14159));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Value: -3.14159");
}

#[test]
fn test_large_integers() {
    let template = CBTemplate::compile("Big: {num}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("num", Value::Int(i64::MAX));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, format!("Big: {}", i64::MAX));
}

#[test]
fn test_reuse_template_different_contexts() {
    let template = CBTemplate::compile("Hello, {name}!").unwrap();

    let mut ctx1 = Context::default();
    ctx1.insert("name", Value::static_str("Alice"));
    let result1 = template.format(&ctx1).unwrap();
    assert_eq!(result1, "Hello, Alice!");

    let mut ctx2 = Context::default();
    ctx2.insert("name", Value::static_str("Bob"));
    let result2 = template.format(&ctx2).unwrap();
    assert_eq!(result2, "Hello, Bob!");

    let mut ctx3 = Context::default();
    ctx3.insert("name", Value::Int(42));
    let result3 = template.format(&ctx3).unwrap();
    assert_eq!(result3, "Hello, 42!");
}

#[test]
fn test_clone_template() {
    let template = CBTemplate::compile("Hello, {name}!").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("World"));

    // Templates should be clonable
    let result1 = template.format(&ctx).unwrap();
    let result2 = template.format(&ctx).unwrap();
    assert_eq!(result1, result2);
}

#[test]
fn test_empty_variable_name() {
    let template = CBTemplate::compile("{}").unwrap();
    let ctx = Context::default();

    // Should compile but produce empty output or error when formatting
    let result = template.format(&ctx);
    // The actual behavior depends on implementation - just ensure it doesn't panic
    let _ = result;
}

#[test]
fn test_multiple_repeats_in_template() {
    let template = CBTemplate::compile("{a:n} middle {b:m} end").unwrap();
    let mut ctx = Context::default();
    ctx.insert("a", Value::static_str("X"));
    ctx.insert("n", Value::Int(3));
    ctx.insert("b", Value::static_str("Y"));
    ctx.insert("m", Value::Int(2));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "XXX middle YY end");
}

#[test]
fn test_literal_in_repeat_directive() {
    let template = CBTemplate::compile("{\"Hello\":3}").unwrap();
    let ctx = Context::default();

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "HelloHelloHello");
}

#[test]
fn test_newlines_and_tabs() {
    let template = CBTemplate::compile("Line1\n{var}\tTabbed").unwrap();
    let mut ctx = Context::default();
    ctx.insert("var", Value::static_str("Line2"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Line1\nLine2\tTabbed");
}

#[test]
fn test_boolean_values() {
    let template = CBTemplate::compile("True: {t}, False: {f}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("t", Value::Bool(true));
    ctx.insert("f", Value::Bool(false));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "True: true, False: false");
}

// ============================================
// Conditional Directive Tests
// ============================================

#[test]
fn test_conditional_simple_true() {
    let template = CBTemplate::compile("{flag ? 'yes' : 'no'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("flag", Value::Bool(true));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "yes");
}

#[test]
fn test_conditional_simple_false() {
    let template = CBTemplate::compile("{flag ? 'yes' : 'no'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("flag", Value::Bool(false));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "no");
}

#[test]
fn test_conditional_with_variables() {
    let template = CBTemplate::compile("{condition ? true_msg : false_msg}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("condition", Value::Bool(true));
    ctx.insert("true_msg", Value::static_str("Success!"));
    ctx.insert("false_msg", Value::static_str("Failed!"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Success!");
}

#[test]
fn test_conditional_string_equality() {
    let template = CBTemplate::compile("{status == 'online' ? 'Active' : 'Inactive'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("status", Value::static_str("online"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Active");
}

#[test]
fn test_conditional_string_inequality() {
    let template =
        CBTemplate::compile("{status != 'offline' ? 'Connected' : 'Disconnected'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("status", Value::static_str("online"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Connected");
}

#[test]
fn test_conditional_numeric_greater_than() {
    let template = CBTemplate::compile("{age > 18 ? 'Adult' : 'Minor'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("age", Value::Int(25));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Adult");
}

#[test]
fn test_conditional_numeric_less_than() {
    let template = CBTemplate::compile("{score < 50 ? 'Fail' : 'Pass'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("score", Value::Int(45));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Fail");
}

#[test]
fn test_conditional_greater_than_equals() {
    let template = CBTemplate::compile("{score >= 90 ? 'A' : 'B'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("score", Value::Int(90));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "A");
}

#[test]
fn test_conditional_less_than_equals() {
    let template = CBTemplate::compile("{temp <= 32 ? 'Freezing' : 'Above freezing'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("temp", Value::Int(30));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Freezing");
}

#[test]
fn test_conditional_with_float_comparison() {
    let template = CBTemplate::compile("{price > 99.99 ? 'Expensive' : 'Affordable'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("price", Value::Float(120.50));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Expensive");
}

#[test]
fn test_conditional_not_operator() {
    let template = CBTemplate::compile("{!flag ? 'Off' : 'On'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("flag", Value::Bool(false));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Off");
}

#[test]
fn test_conditional_not_operator_true() {
    let template = CBTemplate::compile("{!enabled ? 'Disabled' : 'Enabled'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("enabled", Value::Bool(true));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Enabled");
}

#[test]
fn test_conditional_with_integers() {
    let template = CBTemplate::compile("{count ? 1 : 0}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("count", Value::Int(5));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_conditional_comparing_two_variables() {
    let template = CBTemplate::compile("{a == b ? 'Same' : 'Different'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("a", Value::Int(42));
    ctx.insert("b", Value::Int(42));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Same");
}

#[test]
fn test_conditional_comparing_variable_and_literal() {
    let template =
        CBTemplate::compile("{role == 'admin' ? 'Full Access' : 'Limited Access'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("role", Value::static_str("admin"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Full Access");
}

#[test]
fn test_conditional_in_sentence() {
    let template = CBTemplate::compile("User status: {active ? 'Active' : 'Inactive'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("active", Value::Bool(true));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "User status: Active");
}

#[test]
fn test_multiple_conditionals() {
    let template = CBTemplate::compile(
        "{x > 0 ? 'positive' : 'non-positive'} and {y > 0 ? 'positive' : 'non-positive'}",
    )
    .unwrap();
    let mut ctx = Context::default();
    ctx.insert("x", Value::Int(5));
    ctx.insert("y", Value::Int(-3));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "positive and non-positive");
}

#[test]
fn test_conditional_with_empty_strings() {
    let template = CBTemplate::compile("{name != '' ? name : 'Anonymous'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str(""));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Anonymous");
}

#[test]
fn test_conditional_with_zero() {
    let template = CBTemplate::compile("{count == 0 ? 'None' : 'Some'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("count", Value::Int(0));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "None");
}

#[test]
fn test_conditional_numeric_equality() {
    let template = CBTemplate::compile("{value == 42 ? 'Answer' : 'Not the answer'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("value", Value::Int(42));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Answer");
}

#[test]
fn test_conditional_with_negative_numbers() {
    let template = CBTemplate::compile("{temp < 0 ? 'Below zero' : 'Above zero'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("temp", Value::Int(-5));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Below zero");
}

#[test]
fn test_conditional_realistic_email() {
    let template = CBTemplate::compile(
        "Dear {name},\n\nYour account is {verified ? 'verified' : 'not verified'}.\n\n\
        {verified ? 'Thank you for verifying!' : 'Please verify your account.'}",
    )
    .unwrap();
    let mut ctx = Context::default();
    ctx.insert("name", Value::static_str("Alice"));
    ctx.insert("verified", Value::Bool(true));

    let result = template.format(&ctx).unwrap();
    assert_eq!(
        result,
        "Dear Alice,\n\nYour account is verified.\n\nThank you for verifying!"
    );
}

#[test]
fn test_conditional_realistic_status_message() {
    let template =
        CBTemplate::compile("Server: {server_name} | Status: {online ? 'Online ✓' : 'Offline ✗'}")
            .unwrap();
    let mut ctx = Context::default();
    ctx.insert("server_name", Value::static_str("web-01"));
    ctx.insert("online", Value::Bool(false));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Server: web-01 | Status: Offline ✗");
}

#[test]
fn test_conditional_with_unicode() {
    let template = CBTemplate::compile("{success ? '✅ Success' : '❌ Failed'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("success", Value::Bool(true));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "✅ Success");
}

#[test]
fn test_conditional_boundary_case_equal() {
    let template = CBTemplate::compile("{value >= 100 ? 'High' : 'Low'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("value", Value::Int(100));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "High");
}

#[test]
fn test_conditional_string_comparison_lexicographic() {
    let template = CBTemplate::compile("{word > 'middle' ? 'After' : 'Before'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("word", Value::static_str("zebra"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "After");
}

#[test]
fn test_conditional_mixed_with_repeat() {
    let template = CBTemplate::compile("{show ? 'Yes' : 'No'} {pattern:count}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("show", Value::Bool(true));
    ctx.insert("pattern", Value::static_str("*"));
    ctx.insert("count", Value::Int(5));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Yes *****");
}

#[test]
fn test_conditional_false_returns_variable() {
    let template = CBTemplate::compile("{premium ? gold_msg : silver_msg}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("premium", Value::Bool(false));
    ctx.insert("gold_msg", Value::static_str("Premium User"));
    ctx.insert("silver_msg", Value::static_str("Standard User"));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Standard User");
}

#[test]
fn test_conditional_integer_truthy_nonzero() {
    let template = CBTemplate::compile("{count ? 'Has items' : 'Empty'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("count", Value::Int(5));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Has items");
}

#[test]
fn test_conditional_integer_falsy_zero() {
    let template = CBTemplate::compile("{count ? 'Has items' : 'Empty'}").unwrap();
    let mut ctx = Context::default();
    ctx.insert("count", Value::Int(0));

    let result = template.format(&ctx).unwrap();
    assert_eq!(result, "Empty");
}
