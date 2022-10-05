use comp_gen::Interpreter;
use std::collections::HashMap;

use crate::lang;

impl Interpreter for lang::NumExpr {
    type Env = HashMap<String, i64>;
    type Res = i64;

    fn eval_with_env(&self, env: &mut Self::Env) -> Self::Res {
        match self {
            lang::NumExpr::Num(i) => *i,
            lang::NumExpr::Id(id) => *env.get(id).expect(&format!("Didn't find {id}")),
            lang::NumExpr::Plus(a, b) => a.eval_with_env(env) + b.eval_with_env(env),
            lang::NumExpr::Sub(a, b) => a.eval_with_env(env) - b.eval_with_env(env),
            lang::NumExpr::Times(a, b) => a.eval_with_env(env) * b.eval_with_env(env),
            lang::NumExpr::Div(a, b) => a.eval_with_env(env) / b.eval_with_env(env),
        }
    }
}

impl Interpreter for lang::BoolExpr {
    type Env = HashMap<String, i64>;
    type Res = bool;

    fn eval_with_env(&self, env: &mut Self::Env) -> Self::Res {
        match self {
            lang::BoolExpr::Bool(b) => *b,
            lang::BoolExpr::Eq(a, b) => a.eval_with_env(env) == b.eval_with_env(env),
            lang::BoolExpr::Lt(a, b) => a.eval_with_env(env) < b.eval_with_env(env),
            lang::BoolExpr::Not(a) => !a.eval_with_env(env),
            lang::BoolExpr::And(a, b) => a.eval_with_env(env) && b.eval_with_env(env),
        }
    }
}

impl Interpreter for lang::Command {
    type Env = HashMap<String, i64>;
    type Res = ();

    fn eval_with_env(&self, env: &mut Self::Env) -> Self::Res {
        match self {
            // lang::Command::Seq(left, right) => {
            //     left.eval_with_env(env);
            //     right.eval_with_env(env);
            // }
            lang::Command::Seq(items) => {
                for it in items {
                    it.eval_with_env(env);
                }
            }
            lang::Command::Assign(var, expr) => {
                let evaled = expr.eval_with_env(env);
                env.entry(var.clone())
                    .and_modify(|e| *e = evaled)
                    .or_insert(evaled);
            }
            lang::Command::If(cond, tbranch, fbranch) => {
                if cond.eval_with_env(env) {
                    tbranch.eval_with_env(env);
                } else {
                    fbranch.eval_with_env(env);
                }
            }
            lang::Command::While(cond, body) => {
                while cond.eval_with_env(env) {
                    body.eval_with_env(env);
                }
            }
            lang::Command::Mov(dest, src) => {
                let evaled = src.eval_with_env(env);
                env.entry(dest.clone())
                    .and_modify(|e| *e = evaled)
                    .or_insert(evaled);
            }
            lang::Command::Add(rd, rn, rm) => {
                let a = rn.eval_with_env(env);
                let b = rm.eval_with_env(env);
                env.entry(rd.clone())
                    .and_modify(|e| *e = a + b)
                    .or_insert(a + b);
            }
            lang::Command::Sub(rd, rn, rm) => {
                let a = rn.eval_with_env(env);
                let b = rm.eval_with_env(env);
                env.entry(rd.clone())
                    .and_modify(|e| *e = a - b)
                    .or_insert(a - b);
            }
            lang::Command::Smull(rd, rn, rm) => {
                let a = rn.eval_with_env(env);
                let b = rm.eval_with_env(env);
                env.entry(rd.clone())
                    .and_modify(|e| *e = a * b)
                    .or_insert(a * b);
            }
            lang::Command::Sdiv(rd, rn, rm) => {
                let a = rn.eval_with_env(env);
                let b = rm.eval_with_env(env);
                env.entry(rd.clone())
                    .and_modify(|e| *e = a / b)
                    .or_insert(a / b);
            }
        }
    }
}
