use comp_gen::{
    ruler,
    ruler::{egg, SynthLanguage},
};
use log::{debug, warn};
use z3::ast::Ast;

use crate::{fuzz::FuzzEquals, lang};

pub trait TypeEquals: ruler::SynthLanguage {
    type T: Eq;
    /// Return the type of `expr` in the domain `Self::T`.
    /// Returns None if the expr is ill-formed.
    fn get_type(expr: &egg::Pattern<Self>) -> Option<Self::T>;

    /// Check if `lhs` and `rhs` have the same type. Requires that both
    /// sides by well-formed.
    fn type_equals(lhs: &egg::Pattern<Self>, rhs: &egg::Pattern<Self>) -> bool {
        if let (Some(lhs_type), Some(rhs_type)) =
            (Self::get_type(lhs), Self::get_type(rhs))
        {
            lhs_type == rhs_type
        } else {
            false
        }
    }
}

#[derive(Eq, Clone, Copy)]
pub enum VecLangType {
    Vector,
    Scalar,
    Variable,
}

impl PartialEq for VecLangType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // if either side is a variable, then true
            (VecLangType::Variable, _) | (_, VecLangType::Variable) => true,

            // if they are the same, true
            (VecLangType::Vector, VecLangType::Vector)
            | (VecLangType::Scalar, VecLangType::Scalar) => true,

            // otherwise false
            _ => false,
        }
    }
}

impl VecLangType {
    fn join(
        &self,
        b: &VecLangType,
        default: VecLangType,
    ) -> Option<VecLangType> {
        match (self, b) {
            (VecLangType::Variable, VecLangType::Variable) => Some(default),
            // same type goes to same type
            (x, y) if x == y => Some(*x),

            // if one side is a variable, copy the type from the defined value
            (VecLangType::Vector, _) | (_, VecLangType::Vector) => {
                Some(VecLangType::Vector)
            }
            (VecLangType::Scalar, _) | (_, VecLangType::Scalar) => {
                Some(VecLangType::Scalar)
            }
        }
    }
}

impl TypeEquals for lang::VecLang {
    type T = VecLangType;

    fn get_type(expr: &egg::Pattern<Self>) -> Option<Self::T> {
        // This is tracking the type of the expression so far
        let mut types: Vec<Option<VecLangType>> = vec![];

        for node in Self::instantiate(expr).as_ref().iter() {
            match node {
                // unops
                lang::VecLang::Sgn([x])
                | lang::VecLang::Sqrt([x])
                | lang::VecLang::Neg([x]) => {
                    types.push(types[usize::from(*x)].and_then(|t| match t {
                        VecLangType::Vector => None,
                        VecLangType::Scalar => Some(t),
                        VecLangType::Variable => Some(VecLangType::Scalar),
                    }))
                }
                lang::VecLang::VecNeg([x])
                | lang::VecLang::VecSqrt([x])
                | lang::VecLang::VecSgn([x]) => {
                    types.push(types[usize::from(*x)].and_then(|t| match t {
                        VecLangType::Vector => Some(t),
                        VecLangType::Scalar => None,
                        VecLangType::Variable => Some(VecLangType::Vector),
                    }))
                }

                // binops
                lang::VecLang::Add([x, y])
                | lang::VecLang::Mul([x, y])
                | lang::VecLang::Minus([x, y])
                | lang::VecLang::Div([x, y])
                | lang::VecLang::Or([x, y])
                | lang::VecLang::And([x, y])
                | lang::VecLang::Lt([x, y]) => {
                    let type_x = &types[usize::from(*x)];
                    let type_y = &types[usize::from(*y)];
                    let t = type_x.and_then(|xt| {
                        type_y.and_then(|yt| xt.join(&yt, VecLangType::Scalar))
                    });
                    types.push(t);
                }

                lang::VecLang::Concat([x, y])
                | lang::VecLang::VecAdd([x, y])
                | lang::VecLang::VecMinus([x, y])
                | lang::VecLang::VecMul([x, y])
                | lang::VecLang::VecDiv([x, y])
                | lang::VecLang::VecMulSgn([x, y]) => {
                    let type_x = &types[usize::from(*x)];
                    let type_y = &types[usize::from(*y)];
                    let t = type_x.and_then(|xt| {
                        type_y.and_then(|yt| xt.join(&yt, VecLangType::Vector))
                    });
                    types.push(t);
                }

                lang::VecLang::Get([_x, _y]) => todo!(),
                lang::VecLang::Ite([_x, _y, _z]) => todo!(),

                // triops
                lang::VecLang::VecMAC([x, y, z]) => {
                    let type_x = &types[usize::from(*x)];
                    let type_y = &types[usize::from(*y)];
                    let type_z = &types[usize::from(*z)];
                    let t = type_x.and_then(|xt| {
                        type_y.and_then(|yt| {
                            xt.join(&yt, VecLangType::Vector).and_then(|xyt| {
                                type_z.and_then(|zt| {
                                    xyt.join(&zt, VecLangType::Vector)
                                })
                            })
                        })
                    });

                    types.push(t)
                }

                // constants
                lang::VecLang::List(_) | lang::VecLang::Vec(_) => {
                    types.push(Some(VecLangType::Vector))
                }
                lang::VecLang::Const(_) => {
                    types.push(Some(VecLangType::Scalar))
                }
                lang::VecLang::Symbol(_) => {
                    types.push(Some(VecLangType::Variable))
                }
            };
        }

        if let Some(x) = types.pop() {
            x
        } else {
            None
        }
    }
}

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
        let type_check = Self::type_equals(lhs, rhs);
        debug!("type check: {lhs} = {rhs}: {type_check}");
        if !type_check {
            return false;
        }

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

            // let fuzz = Self::fuzz_equals(synth, lhs, rhs, false);
            let smt = match solver.check() {
                z3::SatResult::Unsat => true,
                z3::SatResult::Unknown | z3::SatResult::Sat => false,
            };

            // debug!("model: {:?}", solver.get_model());
            debug!("z3 result: {smt}");

            // if smt != fuzz {
            //     log::info!("{lhs} <=> {rhs} Found ya!!!");
            //     let mut cfg = z3::Config::new();
            //     cfg.set_timeout_msec(1000);
            //     let ctx = z3::Context::new(&cfg);
            //     log::info!("smt: {smt}, fuzz: {fuzz}");
            //     log::info!(
            //         "smt formula: {} = {}",
            //         egg_to_z3(&ctx, &Self::instantiate(lhs))
            //             .map(|x| x.to_string())
            //             .unwrap_or("".to_string()),
            //         egg_to_z3(&ctx, &Self::instantiate(rhs))
            //             .map(|x| x.to_string())
            //             .unwrap_or("".to_string())
            //     );
            //     log::info!("{:?}", solver.get_model());
            //     Self::fuzz_equals(synth, lhs, rhs, true);
            // } else {
            //     log::debug!("smt: {smt}, fuzz: {fuzz}")
            // }

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
            lang::VecLang::Vec(inner) if inner.len() == 1 => {
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
            | lang::VecLang::Concat(_)
            | lang::VecLang::VecMulSgn(_) => return None,
        }
    }
    // return the last element
    buf.pop()
}
