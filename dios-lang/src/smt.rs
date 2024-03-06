use comp_gen::{
    ruler,
    ruler::{egg, SynthLanguage},
};
use log::{debug, warn};
use z3::ast::Ast;

use crate::{fuzz::FuzzEquals, lang};

pub trait SmtEquals: ruler::SynthLanguage {
    fn smt_equals(
        synth: &mut ruler::Synthesizer<Self, ruler::Init>,
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
    ) -> bool;
}

impl SmtEquals for lang::VecLang {
    fn smt_equals(
        synth: &mut ruler::Synthesizer<Self, ruler::Init>,
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
    ) -> bool {
        // if the expressions dont have the same type, they can't be equal
        // abort now

        let mut cfg = z3::Config::new();
        cfg.set_timeout_msec(10000);
        let ctx = z3::Context::new(&cfg);
        let mut solver = z3::Solver::new(&ctx);

        let sqrt_fun: z3::FuncDecl = z3::FuncDecl::new(
            &ctx,
            "sqrt",
            &[&z3::Sort::int(&ctx)],
            &z3::Sort::int(&ctx),
        );

        let left =
            egg_to_z3(&ctx, &mut solver, &Self::instantiate(lhs), &sqrt_fun);
        let right =
            egg_to_z3(&ctx, &mut solver, &Self::instantiate(rhs), &sqrt_fun);

        // if we can translate egg to z3 for both lhs, and rhs, then
        // run the z3 solver. otherwise fallback to fuzz_equals
        if let (Some(lexpr), Some(rexpr)) = (&left, &right) {
            debug!("z3 check {} != {}", lexpr, rexpr);

            // check to see if lexpr is NOT equal to rexpr.
            // if we can't find a counter example to this
            // then we know that they are equal.
            solver.assert(&lexpr._eq(rexpr).not());

            let smt = match solver.check() {
                z3::SatResult::Unsat => true,
                z3::SatResult::Unknown | z3::SatResult::Sat => false,
            };

            debug!("z3 result: {smt}");

            smt
        } else {
            warn!("Couldn't translate {lhs} or {rhs} to smt");
            Self::fuzz_equals(synth, lhs, rhs, false)
        }
    }
}

/// Translate an egg::RecExpr into an equivalent z3 expression.
/// This only works for operations on integers for now.
pub fn egg_to_z3<'a>(
    ctx: &'a z3::Context,
    _solver: &mut z3::Solver,
    expr: &egg::RecExpr<lang::VecLang>,
    sqrt_fun: &'a z3::FuncDecl,
) -> Option<z3::ast::Int<'a>> {
    // This translate works by walking through the RecExpr vector
    // in order, and translating just that node. We push this
    // translation into a buffer as we go. Any time we need to
    // reference a child, we can look up the corresponding index
    // in the buffer. This works because we have the guarantee that
    // some element i in RecExpr only ever refers to elements j < i.
    // By the time we need them, we will have already translated them.

    let mut buf: Vec<z3::ast::Int> = vec![];

    for node in expr.as_ref().iter() {
        match node {
            lang::VecLang::Const(lang::Value::Int(i)) => {
                buf.push(z3::ast::Int::from_i64(ctx, *i as i64))
            }
            lang::VecLang::Symbol(v) => {
                buf.push(z3::ast::Int::new_const(ctx, v.to_string()))
            }
            lang::VecLang::Add([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int + y_int);
            }
            lang::VecLang::Mul([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int * y_int);
            }
            lang::VecLang::Minus([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int - y_int);
            }
            lang::VecLang::Div([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                // let zero = z3::ast::Int::from_i64(ctx, 0);
                // solver.assert(&y_int._eq(&zero).not());
                buf.push(x_int / y_int);
            }
            lang::VecLang::SqrtSgn([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];

                let sqrt_int = sqrt_fun
                    .apply(&[x_int])
                    .as_int()
                    .expect("z3 Sqrt didn't return an int.");

                let zero = z3::ast::Int::from_i64(ctx, 0);
                let one = z3::ast::Int::from_i64(ctx, 1);
                let m_one = z3::ast::Int::from_i64(ctx, -1);

                let inner: z3::ast::Int = (y_int.gt(&zero)).ite(&m_one, &one);
                let neg_sgn = (x_int._eq(&zero)).ite(&zero, &inner);

                buf.push(sqrt_int * neg_sgn)
            }
            lang::VecLang::Neg([x]) => {
                let x_int = &buf[usize::from(*x)];
                buf.push(-x_int);
            }
            lang::VecLang::Sgn([x]) => {
                let x_int = &buf[usize::from(*x)];
                let zero = z3::ast::Int::from_i64(ctx, 0);
                let one = z3::ast::Int::from_i64(ctx, 1);
                let m_one = z3::ast::Int::from_i64(ctx, -1);

                let inner: z3::ast::Int = (x_int.gt(&zero)).ite(&one, &m_one);
                let sgn = (x_int._eq(&zero)).ite(&zero, &inner);

                buf.push(sgn);
            }
            lang::VecLang::Sqrt([x]) => {
                let x_int = &buf[usize::from(*x)];
                buf.push(
                    sqrt_fun
                        .apply(&[x_int])
                        .as_int()
                        .expect("z3 Sqrt didn't return an int."),
                );
            }

            // an attempt to support vector operators.
            // I think I should just be able to map them
            // on to their scalar counter parts.
            lang::VecLang::VecAdd([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int + y_int);
            }
            lang::VecLang::VecMinus([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int - y_int);
            }
            lang::VecLang::VecMul([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int * y_int);
            }
            lang::VecLang::VecDiv([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int / y_int);
            }
            lang::VecLang::VecMulSgn([x, y]) => {
                // grab smt versions of inputs
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];

                // make a zero constant
                let zero = z3::ast::Int::from_i64(ctx, 0);

                // if x > 0 { y } else { -y }
                let inner: z3::ast::Int = (x_int.gt(&zero)).ite(y_int, &-y_int);
                // if x == 0 { 0 } else { inner }
                let sgn = (x_int._eq(&zero)).ite(&zero, &inner);

                buf.push(sgn);
            }
            lang::VecLang::VecNeg([x]) => {
                let x_int = &buf[usize::from(*x)];
                buf.push(-x_int);
            }
            lang::VecLang::VecMAC([acc, x, y]) => {
                let acc_int = &buf[usize::from(*acc)];
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push((x_int * y_int) + acc_int);
            }
            lang::VecLang::VecMULS([acc, x, y]) => {
                let acc_int = &buf[usize::from(*acc)];
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(acc_int - (x_int * y_int));
            }
            lang::VecLang::VecSgn([x]) => {
                let x_int = &buf[usize::from(*x)];
                let zero = z3::ast::Int::from_i64(ctx, 0);
                let one = z3::ast::Int::from_i64(ctx, 1);
                let m_one = z3::ast::Int::from_i64(ctx, -1);

                let inner: z3::ast::Int = (x_int.gt(&zero)).ite(&m_one, &one);
                let sgn = (x_int._eq(&zero)).ite(&zero, &inner);

                buf.push(sgn);
            }
            lang::VecLang::VecSqrt([x]) => {
                let x_int = &buf[usize::from(*x)];
                buf.push(
                    sqrt_fun
                        .apply(&[x_int])
                        .as_int()
                        .expect("z3 Sqrt didn't return an int."),
                );
            }
            lang::VecLang::VecSqrtSgn([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];

                let sqrt_int = sqrt_fun
                    .apply(&[x_int])
                    .as_int()
                    .expect("z3 Sqrt didn't return an int.");

                let zero = z3::ast::Int::from_i64(ctx, 0);
                let one = z3::ast::Int::from_i64(ctx, 1);
                let m_one = z3::ast::Int::from_i64(ctx, -1);

                let inner: z3::ast::Int = (y_int.gt(&zero)).ite(&m_one, &one);
                let neg_sgn = (x_int._eq(&zero)).ite(&zero, &inner);

                buf.push(sqrt_int * neg_sgn)
            }
            lang::VecLang::Vec(inner) if inner.len() == 1 => {
                let x = inner[0];
                let x_int = &buf[usize::from(x)];
                buf.push(x_int.clone());
            }
            lang::VecLang::LitVec(inner) if inner.len() == 1 => {
                let x = inner[0];
                let x_int = &buf[usize::from(x)];
                buf.push(x_int.clone());
            }
            // unsupported operations
            // return early
            lang::VecLang::Const(_)
            | lang::VecLang::Or(_)
            | lang::VecLang::And(_)
            | lang::VecLang::Ite(_)
            | lang::VecLang::Lt(_)
            | lang::VecLang::List(_)
            | lang::VecLang::Get(_)
            | lang::VecLang::Vec(_)
            | lang::VecLang::LitVec(_)
            | lang::VecLang::Concat(_)
            | lang::VecLang::Let(_) => return None,
        }
    }
    // return the last element
    buf.pop()
}
