use std::convert::TryInto;
use std::fs;
use z3::ast;
use z3::ast::Ast;
extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;
use pest::Parser;
use pest::{iterators::*, prec_climber::*};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "grammer.pest"]
struct LangParser;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(shl, Left) | Operator::new(shr, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left),
        ])
    };
}

fn eval<'a>(
    ctx: &'a z3::Context,
    expression: Pairs<Rule>,
    env: &mut HashMap<String, ast::BV<'a>>,
) -> ast::BV<'a> {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::num => ast::BV::from_i64(&ctx, pair.as_str().parse::<i64>().unwrap(), 8),
            Rule::expr => eval(ctx, pair.into_inner(), env),
            Rule::var | Rule::hole => {
                return env
                    .entry(pair.as_str().to_string())
                    .or_insert(z3::ast::BV::new_const(&ctx, pair.as_str(), 8))
                    .clone();
            }
            _ => unreachable!(),
        },
        |lhs: ast::BV<'a>, op: Pair<Rule>, rhs: ast::BV<'a>| match op.as_rule() {
            Rule::add => lhs.bvadd(&rhs),
            Rule::subtract => lhs.bvsub(&rhs),
            Rule::multiply => lhs.bvmul(&rhs),
            Rule::divide => lhs.bvsdiv(&rhs),
            Rule::shr => lhs.bvashr(&rhs),
            Rule::shl => lhs.bvshl(&rhs),
            _ => unreachable!(),
        },
    )
}

fn main() {
    let config = z3::Config::new();
    let ctx = z3::Context::new(&config);
    let solver = z3::Solver::new(&ctx);

    let raw_input = fs::read_to_string("hole.txt").expect("cannot read file");
    let mut lines = raw_input.lines();
    let line_without_holes = lines.next().expect("holeless line");
    let line_with_holes = lines.next().expect("hole line");
    let successful_parse = LangParser::parse(Rule::calculation, line_without_holes)
        .unwrap_or_else(|e| panic!("{}", e));
    let mut vars = HashMap::new();

    let expr1 = eval(&ctx, successful_parse, &mut vars);
    let successful_parse2 =
        LangParser::parse(Rule::calculation, line_with_holes).unwrap_or_else(|e| panic!("{}", e));

    let expr2 = eval(&ctx, successful_parse2, &mut vars);

    let plain_vars: Vec<ast::Dynamic> = vars
        .iter()
        .filter_map(|(k, v)| {
            if !k.starts_with("?") {
                Some(v.clone().into())
            } else {
                None
            }
        })
        .collect();

    let res: z3::ast::Bool = z3::ast::forall_const(
        &ctx,
        &plain_vars.iter().collect::<Vec<_>>(),
        &[],
        &expr1._eq(&expr2).clone().into(),
    )
    .try_into()
    .unwrap();
    println!("{}", res);
    solver.assert(&res);
    solver.check();
    let model = solver.get_model();
    println!("{}", model);
}
