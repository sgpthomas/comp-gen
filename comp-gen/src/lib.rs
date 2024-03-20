mod compiler;
pub mod config;
mod cost;
mod phases;
mod run;
mod stats;

// use ruler::egg;
use egg;

pub use compiler::Compiler;
pub use cost::{CostMetric, CostMetrics};
pub use phases::PhaseBuilder;
// pub use ruler::{
//     letter, map, self_product, CVec, Equality, Init, Report, SynthAnalysis,
//     SynthLanguage, SynthParams, Synthesizer, Uninit,
// };

pub trait Interpreter {
    type Env: Default;
    type Res;

    fn eval_with_env(&self, env: &mut Self::Env) -> Self::Res;
    fn eval(&self) -> Self::Res {
        self.eval_with_env(&mut Self::Env::default())
    }
}

/// Recursively translate some type `T` into a `egg::RecExpr`.
pub trait ToRecExpr<T> {
    fn to_recexpr(&self, expr: &mut egg::RecExpr<T>) -> egg::Id;
}

/// A trait that specifies how an `egg::PatternAst` can be converted
/// into an `egg::RecExpr`.
pub trait FromPattern: Sized {
    fn from_pattern(pat: &egg::PatternAst<Self>) -> egg::RecExpr<Self>;
}
