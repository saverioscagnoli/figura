use figura::Context;
use figura::Template;
use figura::Value;

fn main() {
    let mut ctx = Context::default();

    ctx.insert("company", Value::static_str("Acme Corp"));
    ctx.insert("quarter", Value::static_str("Q4 2024"));
    ctx.insert("revenue", Value::Int(1250000));
    ctx.insert("expenses", Value::Int(890000));
    ctx.insert("profit", Value::Int(360000));
    ctx.insert("growth", Value::Float(12.5));
    ctx.insert("target_met", Value::Bool(true));
    ctx.insert("employees", Value::Int(42));
    ctx.insert("bar_width", Value::Int(40));

    let template = Template::<'{', '}'>::compile(
        "\
{'=':70}
{' ':20}{company} - Quarterly Report
{' ':25}{quarter}
{'=':70}

FINANCIAL SUMMARY
{'-':70}
Revenue:        ${revenue}
Expenses:       ${expenses}
Profit:         ${profit}
Growth Rate:    {growth}%

Performance:    {target_met ? '✓ TARGET MET' : '✗ TARGET MISSED'}
Status:         {profit > 0 ? 'PROFITABLE' : 'LOSS'}

METRICS
{'-':70}
Employees:      {employees}
Profit/Employee: ${profit != 0 ? '8571' : '0'}

VISUAL BREAKDOWN
{'-':70}
Revenue:  {'█':bar_width}
Expenses: {'█':28}
Profit:   {'█':12}

GROWTH INDICATOR
{'-':70}
{growth > 10.0 ? '↑ Strong Growth (★★)' : '→ Moderate Growth (★)'}

{'=':70}
Report Generated Successfully
{'=':70}
",
    )
    .unwrap();

    println!("{}", template.format(&ctx).unwrap());
}
