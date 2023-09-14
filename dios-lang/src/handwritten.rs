use egg::*;

use crate::lang::VecLang;

pub fn build_litvec_rule(vec_width: usize) -> Vec<egg::Rewrite<VecLang, ()>> {
    let mut gets: Vec<String> = Vec::with_capacity(vec_width);
    for i in 0..vec_width {
        gets.push(format!("(Get ?a ?i{})", i))
    }
    let all_gets = gets.join(" ");

    let searcher: egg::Pattern<VecLang> =
        format!("(Vec {})", all_gets).parse().unwrap();
    let applier: egg::Pattern<VecLang> =
        format!("(LitVec {})", all_gets).parse().unwrap();

    let mut res = vec![egg::rewrite!("litvec"; { searcher } => { applier })];

    // add rules of the form (Vec 0 (get ?a ?n0) ..) => (VecLit 0 (get ?a ?n0) ..)
    for lit in 0..1 {
        for w in 0..vec_width {
            let mut all_gets_search = gets.clone();
            let mut all_gets_apply = gets.clone();
            all_gets_search[w] = lit.to_string();
            all_gets_apply[w] = lit.to_string();
            let searcher: egg::Pattern<VecLang> =
                format!("(Vec {})", all_gets_search.join(" "))
                    .parse()
                    .unwrap();
            let applier: egg::Pattern<VecLang> =
                format!("(LitVec {})", all_gets_apply.join(" "))
                    .parse()
                    .unwrap();
            let name = format!("litvec-{lit}-{w}");
            res.push(egg::rewrite!(name; { searcher } => { applier }))
        }
    }

    res
}
