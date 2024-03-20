use std::collections::HashMap;

use egg;

use crate::lang;

pub struct Environment<T> {
    /// Maps prefixes to the numbers of times they have been used
    /// so that we can generate meaningful but unique names
    map: HashMap<String, usize>,

    env: Vec<(String, T)>,
}

impl<T> Default for Environment<T> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            env: Vec::default(),
        }
    }
}

impl<T> Environment<T> {
    fn bind<S: ToString>(&mut self, prefix: S, value: T) -> String {
        let count = self
            .map
            .entry(prefix.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(0);
        let name = format!("{}{count}", prefix.to_string());
        self.env.push((name.to_string(), value));
        name
    }

    #[allow(unused)]
    fn env_iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.env.iter().map(|(name, val)| (name.as_str(), val))
    }
}

pub trait Letify: Sized {
    type Data;
    fn letify_with_env(self, env: &mut Environment<Self::Data>) -> Self;
    fn reify_env(self, env: Environment<Self::Data>) -> Self;

    fn letify(self) -> Self {
        let mut env = Environment::default();
        self.letify_with_env(&mut env).reify_env(env)
    }
}

// impl Letify for lang::VecAst {
//     type Data = lang::VecAst;

//     fn letify_with_env(self, env: &mut Environment<Self::Data>) -> Self {
//         match self {
//             lang::VecAst::Add(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("add_l", lval);
//                 let rvar = env.bind("add_r", rval);
//                 lang::VecAst::Add(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::Mul(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("mul_l", lval);
//                 let rvar = env.bind("mul_r", rval);
//                 lang::VecAst::Mul(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::Minus(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("minus_l", lval);
//                 let rvar = env.bind("minus_r", rval);
//                 lang::VecAst::Minus(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::Div(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("div_l", lval);
//                 let rvar = env.bind("div_r", rval);
//                 lang::VecAst::Div(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::Or(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("or_l", lval);
//                 let rvar = env.bind("or_r", rval);
//                 lang::VecAst::Or(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::And(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("and_l", lval);
//                 let rvar = env.bind("and_r", rval);
//                 lang::VecAst::And(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::Ite(_c, _t, _f) => todo!(),
//             lang::VecAst::Lt(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("lt_l", lval);
//                 let rvar = env.bind("lt_r", rval);
//                 lang::VecAst::Lt(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::SqrtSgn(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("sqrtsgn_l", lval);
//                 let rvar = env.bind("sqrtsgn_r", rval);
//                 lang::VecAst::SqrtSgn(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::Concat(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("concat_l", lval);
//                 let rvar = env.bind("concat_r", rval);
//                 lang::VecAst::Concat(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::Get(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("get_l", lval);
//                 let rvar = env.bind("get_r", rval);
//                 lang::VecAst::Get(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::Sqrt(x) => {
//                 let xval = x.letify_with_env(env);
//                 let xvar = env.bind("sqrt_x", xval);
//                 lang::VecAst::Sqrt(Box::new(lang::VecAst::Symbol(xvar)))
//             }
//             lang::VecAst::Sgn(x) => {
//                 let xval = x.letify_with_env(env);
//                 let xvar = env.bind("sgn_x", xval);
//                 lang::VecAst::Sgn(Box::new(lang::VecAst::Symbol(xvar)))
//             }
//             lang::VecAst::Neg(x) => {
//                 let xval = x.letify_with_env(env);
//                 let xvar = env.bind("neg_x", xval);
//                 lang::VecAst::Neg(Box::new(lang::VecAst::Symbol(xvar)))
//             }
//             x @ lang::VecAst::List(_) => x,
//             x @ lang::VecAst::Vec(_) => x,
//             x @ lang::VecAst::LitVec(_) => x,
//             lang::VecAst::VecAdd(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("vecadd_l", lval);
//                 let rvar = env.bind("vecadd_r", rval);
//                 lang::VecAst::VecAdd(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::VecMul(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("vecmul_l", lval);
//                 let rvar = env.bind("vecmul_r", rval);
//                 lang::VecAst::VecMul(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::VecMinus(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("vecminus_l", lval);
//                 let rvar = env.bind("vecminus_r", rval);
//                 lang::VecAst::VecMinus(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::VecDiv(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("vecdiv_l", lval);
//                 let rvar = env.bind("vecdiv_r", rval);
//                 lang::VecAst::VecDiv(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::VecMulSgn(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("vecmulsgn_l", lval);
//                 let rvar = env.bind("vecmulsgn_r", rval);
//                 lang::VecAst::VecMulSgn(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::VecSqrtSgn(l, r) => {
//                 let lval = l.letify_with_env(env);
//                 let rval = r.letify_with_env(env);
//                 let lvar = env.bind("vecsqrtsgn_l", lval);
//                 let rvar = env.bind("vecsqrtsgn_r", rval);
//                 lang::VecAst::VecSqrtSgn(
//                     Box::new(lang::VecAst::Symbol(lvar)),
//                     Box::new(lang::VecAst::Symbol(rvar)),
//                 )
//             }
//             lang::VecAst::VecNeg(x) => {
//                 let xval = x.letify_with_env(env);
//                 let xvar = env.bind("vecneg_x", xval);
//                 lang::VecAst::VecNeg(Box::new(lang::VecAst::Symbol(xvar)))
//             }
//             lang::VecAst::VecSqrt(x) => {
//                 let xval = x.letify_with_env(env);
//                 let xvar = env.bind("vecsqrt_x", xval);
//                 lang::VecAst::VecSqrt(Box::new(lang::VecAst::Symbol(xvar)))
//             }
//             lang::VecAst::VecSgn(x) => {
//                 let xval = x.letify_with_env(env);
//                 let xvar = env.bind("vecsgn_x", xval);
//                 lang::VecAst::VecSgn(Box::new(lang::VecAst::Symbol(xvar)))
//             }
//             lang::VecAst::VecMAC(a, b, c) => {
//                 let aval = a.letify_with_env(env);
//                 let bval = b.letify_with_env(env);
//                 let cval = c.letify_with_env(env);
//                 let avar = env.bind("vecmac_a", aval);
//                 let bvar = env.bind("vecmac_b", bval);
//                 let cvar = env.bind("vecmac_c", cval);
//                 lang::VecAst::VecMAC(
//                     Box::new(lang::VecAst::Symbol(avar)),
//                     Box::new(lang::VecAst::Symbol(bvar)),
//                     Box::new(lang::VecAst::Symbol(cvar)),
//                 )
//             }
//             lang::VecAst::VecMULS(a, b, c) => {
//                 let aval = a.letify_with_env(env);
//                 let bval = b.letify_with_env(env);
//                 let cval = c.letify_with_env(env);
//                 let avar = env.bind("vecmuls_a", aval);
//                 let bvar = env.bind("vecmuls_b", bval);
//                 let cvar = env.bind("vecmuls_c", cval);
//                 lang::VecAst::VecMULS(
//                     Box::new(lang::VecAst::Symbol(avar)),
//                     Box::new(lang::VecAst::Symbol(bvar)),
//                     Box::new(lang::VecAst::Symbol(cvar)),
//                 )
//             }
//             x @ lang::VecAst::Let(..) => x,
//             x @ lang::VecAst::Const(_) => x,
//             x @ lang::VecAst::Symbol(_) => x,
//         }
//     }

//     fn reify_env(self, env: Environment<Self::Data>) -> Self {
//         lang::VecAst::List(
//             env.env
//                 .into_iter()
//                 .map(|(name, val)| lang::VecAst::Let(name, Box::new(val)))
//                 .chain(vec![self].into_iter())
//                 .collect(),
//         )
//     }
// }

// impl Letify for egg::RecExpr<lang::VecLang> {
//     type Data = ();
//     fn letify_with_env(self, _: &mut Environment<Self::Data>) -> Self {
//         self
//     }

//     fn reify_env(self, _env: Environment<Self::Data>) -> Self {
//         self
//     }

//     fn letify(self) -> Self {
//         lang::VecAst::from(self).letify().into()
//     }
// }
