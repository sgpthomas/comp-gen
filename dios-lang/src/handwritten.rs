use comp_gen::ruler::egg;
use itertools::Itertools;

use crate::lang::{Value, VecLang};

pub fn build_litvec_rule(vec_width: usize) -> egg::Rewrite<VecLang, ()> {
    let mem_vars = ids_with_prefix(&"a".to_string(), vec_width);
    let mut gets: Vec<String> = Vec::with_capacity(vec_width);
    for i in 0..vec_width {
        gets.push(format!("(Get {} ?{}{})", mem_vars[i], "i", i))
    }
    let all_gets = gets.join(" ");

    let searcher: egg::Pattern<VecLang> =
        format!("(Vec {})", all_gets).parse().unwrap();
    let applier: egg::Pattern<VecLang> =
        format!("(LitVec {})", all_gets).parse().unwrap();

    egg::rewrite!("litvec"; { searcher } => { applier }
        if is_all_same_memory_or_zero(&mem_vars))
}

/// Returns a function that checks to see if the accesses in a vector
/// are all accesses to the same memory or zero.
fn is_all_same_memory_or_zero(
    vars: &Vec<String>,
) -> impl Fn(&mut egg::EGraph<VecLang, ()>, egg::Id, &egg::Subst) -> bool {
    let vars: Vec<egg::Var> = vars.iter().map(|v| v.parse().unwrap()).collect();
    let zero = VecLang::Const(Value::Int(0));
    move |egraph, _, subst| {
        let non_zero_gets = vars
            .iter()
            .filter(|v| !egraph[subst[**v]].nodes.contains(&zero))
            .unique_by(|v| egraph.find(subst[**v]));
        non_zero_gets.count() < 2
    }
}

/// Generate `count` ids with starting with the prefix `pre`
fn ids_with_prefix(pre: &String, count: usize) -> Vec<String> {
    let mut ids: Vec<String> = Vec::with_capacity(count);
    for i in 0..count {
        ids.push(format!("?{}{}", pre, i))
    }
    ids
}
