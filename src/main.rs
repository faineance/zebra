use std::convert::TryInto;
use std::fs;
use z3::ast::Ast;
extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;
use pest::Parser;

use pest::{iterators::*, prec_climber::*};

#[derive(Parser)]
#[grammar = "grammer.pest"] // relative to src
struct LangParser;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left),
            Operator::new(power, Right),
        ])
    };
}

fn eval(expression: Pairs<Rule>) -> f64 {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::num => pair.as_str().parse::<f64>().unwrap(),
            Rule::expr => eval(pair.into_inner()),
            Rule::var => {
                println!("{}", pair.as_str());
                unimplemented!()
            }
            _ => unreachable!(),
        },
        |lhs: f64, op: Pair<Rule>, rhs: f64| match op.as_rule() {
            Rule::add => lhs + rhs,
            Rule::subtract => lhs - rhs,
            Rule::multiply => lhs * rhs,
            Rule::divide => lhs / rhs,
            Rule::power => lhs.powf(rhs),
            _ => unreachable!(),
        },
    )
}

fn main() {
    let unparsed_file = fs::read_to_string("hole.txt").expect("cannot read file");

    let successful_parse =
        LangParser::parse(Rule::calculation, &unparsed_file).unwrap_or_else(|e| panic!("{}", e));
    println!("{}", &successful_parse);
    println!("{}", eval(successful_parse));

    let config = z3::Config::new();
    let ctx = z3::Context::new(&config);
    let solver = z3::Solver::new(&ctx);

    let x = z3::ast::BV::new_const(&ctx, "x", 8);
    let y = z3::ast::BV::new_const(&ctx, "y", 8);
    let h = z3::ast::BV::new_const(&ctx, "h", 8);

    let phi_s = y._eq(&x.bvshl(&h));
    let phi_n = y._eq(&x.bvmul(&z3::ast::BV::from_i64(&ctx, 4, 8)));

    let res: z3::ast::Bool = z3::ast::forall_const(
        &ctx,
        &[&x.clone().into(), &y.clone().into()],
        &[],
        &phi_s._eq(&phi_n).clone().into(),
    )
    .try_into()
    .unwrap();

    solver.assert(&res);
    solver.check();
    let model = solver.get_model();
    println!("{}", model);
}
