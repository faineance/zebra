use std::convert::TryInto;
use z3::ast::Ast;

fn main() {
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
