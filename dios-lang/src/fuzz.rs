use std::collections::HashMap;

use comp_gen::{
    ruler,
    ruler::{egg, SynthLanguage},
};
use itertools::Itertools;

use crate::{
    lang,
    synthesis::{split_into_halves, vecs_eq},
};

pub trait FuzzEquals: ruler::SynthLanguage {
    fn fuzz_equals(
        synth: &mut ruler::Synthesizer<Self, ruler::Init>,
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
        _debug: bool,
    ) -> bool;
}

impl FuzzEquals for lang::VecLang {
    fn fuzz_equals(
        synth: &mut ruler::Synthesizer<Self, ruler::Init>,
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
        _debug: bool,
    ) -> bool {
        // let n = synth.params.num_fuzz;
        // let n = 10;
        let mut env = HashMap::default();

        let lhs_vars = lhs.vars();
        let lhs_sorted = lhs_vars.iter().sorted().collect_vec();
        let rhs_vars = rhs.vars();
        let rhs_sorted = rhs_vars.iter().sorted().collect_vec();

        // TODO: I'm not sure that I actually want this constraint?
        // won't this prevent rules like (+ a a) => 0? Though I already
        // prevent that with variable duplication
        if lhs_sorted != rhs_sorted {
            log::debug!(
                "lhs vars != rhs vars: {:?} != {:?}",
                lhs_sorted,
                rhs_sorted
            );
            return false;
        }

        for var in lhs.vars() {
            env.insert(var, vec![]);
        }

        for var in rhs.vars() {
            env.insert(var, vec![]);
        }

        // eprintln!("env: {env:?}");

        let (_n_ints, n_vecs) = split_into_halves(10);
        // let (n_neg_ints, n_pos_ints) = split_into_halves(n_ints);

        let possibilities = vec![-10, -5, -2, -1, 0, 1, 2, 5, 10];
        // let possibilities = vec![0, 1, 2];
        for perms in possibilities.iter().permutations(env.keys().len()) {
            for (i, cvec) in perms.iter().zip(env.values_mut()) {
                cvec.push(Some(lang::Value::Int(**i)));
            }
        }

        // add some random vectors
        let mut length = 0;
        for cvec in env.values_mut() {
            cvec.extend(
                lang::Value::sample_vec(
                    &mut synth.rng,
                    -100,
                    100,
                    synth.lang_config.vector_size,
                    n_vecs,
                )
                .into_iter()
                .map(Some),
            );
            length = cvec.len();
        }

        let lvec = Self::eval_pattern(lhs, &env, length);
        let rvec = Self::eval_pattern(rhs, &env, length);
        // let debug = false;
        // if lvec != rvec || debug {
        //     debug!(
        //         "  env: {:?}",
        //         env.into_iter()
        //             .map(|(v, env)| (
        //                 v,
        //                 env.into_iter()
        //                     .flat_map(|x| match x {
        //                         Some(lang::Value::Vec(v)) =>
        //                             Some(lang::Value::Vec(v)),
        //                         _ => None,
        //                     })
        //                     .collect::<Vec<_>>()
        //             ))
        //             .collect::<Vec<_>>()
        //     );
        //     debug!("  lhs: {lhs}");
        //     debug!("  rhs: {rhs}");
        //     debug!("  lvec: {:?}", lvec.iter().flatten().collect_vec(),);
        //     debug!("  rvec: {:?}", rvec.iter().flatten().collect_vec());
        //     debug!("  eq: {}", vecs_eq(&lvec, &rvec));
        // }

        vecs_eq(&lvec, &rvec)
    }
}
