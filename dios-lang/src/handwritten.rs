use comp_gen::ruler::egg;

use crate::lang::VecLang;

pub fn build_litvec_rule(vec_width: usize) -> egg::Rewrite<VecLang, ()> {
    // let mem_vars = ids_with_prefix(&"a".to_string(), vec_width);
    let mut gets: Vec<String> = Vec::with_capacity(vec_width);
    for i in 0..vec_width {
        gets.push(format!("(Get ?a ?{}{})", "i", i))
    }
    let all_gets = gets.join(" ");

    let searcher: egg::Pattern<VecLang> =
        format!("(Vec {})", all_gets).parse().unwrap();
    let applier: egg::Pattern<VecLang> =
        format!("(LitVec {})", all_gets).parse().unwrap();

    egg::rewrite!("litvec"; { searcher } => { applier })
}
