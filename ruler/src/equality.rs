use crate::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Definition of an equality.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "SerializedEq")]
#[serde(into = "SerializedEq")]
#[serde(bound = "L: SynthLanguage")]
pub struct Equality<L: SynthLanguage> {
    pub name: Arc<str>,
    pub lhs: Pattern<L>,
    pub ids: Option<(Id, Id)>,
    pub rhs: Pattern<L>,
    pub rewrites: Vec<Rewrite<L, SynthAnalysis>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SerializedEq {
    lhs: String,
    rhs: String,
    bidirectional: bool,
}

impl<L: SynthLanguage + 'static> From<SerializedEq> for Equality<L> {
    fn from(ser: SerializedEq) -> Self {
        let lhs: Pattern<L> = ser.lhs.parse().unwrap();
        let rhs: Pattern<L> = ser.rhs.parse().unwrap();
        let lhs = L::instantiate(&lhs);
        let rhs = L::instantiate(&rhs);
        Self::new(&lhs, &rhs).unwrap()
    }
}

impl<L: SynthLanguage> From<Equality<L>> for SerializedEq {
    fn from(eq: Equality<L>) -> Self {
        Self {
            lhs: eq.lhs.to_string(),
            rhs: eq.rhs.to_string(),
            bidirectional: eq.rewrites.len() > 1,
        }
    }
}

impl<L: SynthLanguage> Hash for Equality<L> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<L: SynthLanguage> Display for Equality<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<L: SynthLanguage> PartialEq for Equality<L> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<L: SynthLanguage> Eq for Equality<L> {}

struct NotUndefined<L: SynthLanguage> {
    name: String,
    rhs: Pattern<L>,
}

impl<L: SynthLanguage> Applier<L, SynthAnalysis> for NotUndefined<L> {
    fn vars(&self) -> Vec<Var> {
        self.rhs.vars()
    }

    fn apply_one(
        &self,
        egraph: &mut EGraph<L, SynthAnalysis>,
        matched_id: Id,
        subst: &Subst,
        searcher_ast: Option<&PatternAst<L>>,
        rule_name: Symbol,
    ) -> Vec<Id> {
        if !egraph[matched_id].data.is_defined() {
            return vec![];
        }
        let ids = self.rhs.apply_one(
            egraph,
            matched_id,
            subst,
            searcher_ast,
            rule_name,
        );

        if ids.is_empty() {
            return vec![];
        }

        let id = ids[0];
        if !egraph[id].data.is_defined() {
            return vec![];
        }

        for (i, (a, b)) in egraph[matched_id]
            .data
            .cvec
            .iter()
            .zip(&egraph[id].data.cvec)
            .enumerate()
        {
            match (a, b) {
                (Some(a), Some(b)) if a != b => {
                    for class in egraph.classes() {
                        if let Some(var) =
                            class.nodes.iter().find_map(|n| n.to_var())
                        {
                            if let Some(Some(val)) =
                                class.data.cvec.get(i).as_ref()
                            {
                                eprintln!("  {} = {}", var, val);
                            } else {
                                eprintln!("  {} = none", var);
                            }
                        }
                    }

                    // let mut extractor = Extractor::new(
                    //     &egraph,
                    //     RandomCost {
                    //         rng: Pcg64::seed_from_u64(0),
                    //     },
                    // );
                    // for i in 0..10 {
                    //     let (_best_cost, best) = extractor.find_best(id);
                    //     log::info!("{}: {}", i, best.pretty(80));
                    // }
                    assert_eq!(a, b, "bad rule {}", self.name)
                }
                _ => (),
            }
        }

        // we have to manually union now
        for id in &ids {
            egraph.union(matched_id, *id);
        }

        ids
    }
}

struct RandomCost {
    rng: Pcg32,
}
impl<L: SynthLanguage> CostFunction<L> for RandomCost {
    type Cost = i32;

    fn cost<C>(&mut self, enode: &L, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        let r = self.rng.gen_range(0, 25);
        enode.fold(r, |sum, id| sum + costs(id))
        // todo!()
    }
}

impl<L: SynthLanguage> Equality<L> {
    /// Create a new [Equality] from two [RecExprs](https://docs.rs/egg/0.6.0/egg/struct.RecExpr.html).
    pub fn new<'a>(
        mut e1: &'a RecExpr<L>,
        mut e2: &'a RecExpr<L>,
    ) -> Option<Self> {
        if e1 < e2 {
            std::mem::swap(&mut e1, &mut e2);
        }
        let mut forward: (
            String,
            Pattern<L>,
            Pattern<L>,
            Option<Rewrite<L, _>>,
        ) = {
            let map = &mut HashMap::default();
            let lhs = L::generalize(&e1, map);
            let rhs = L::generalize(&e2, map);
            // HACK: we need to manually remove quotes from the produced
            // string because the `symbolic_expression` library that `egg`
            // uses to display s-expressions adds them in a way that's very
            // hard to remove without doing this hack.
            let name = format!("{} => {}", lhs, rhs).replace("\"", "");
            let defined_rhs = NotUndefined {
                name: name.clone(),
                rhs: rhs.clone(),
            };
            (
                name.clone(),
                lhs.clone(),
                rhs.clone(),
                Rewrite::new(name, lhs.clone(), defined_rhs).ok(),
            )
        };

        let mut back: (String, Pattern<L>, Pattern<L>, Option<Rewrite<L, _>>) = {
            let map = &mut HashMap::default();
            let lhs = L::generalize(&e2, map);
            let rhs = L::generalize(&e1, map);
            // HACK: same hack as above
            let name = format!("{} => {}", lhs, rhs).replace("\"", "");
            let defined_rhs = NotUndefined {
                name: name.clone(),
                rhs: rhs.clone(),
            };
            (
                name.clone(),
                lhs.clone(),
                rhs.clone(),
                Rewrite::new(name, lhs.clone(), defined_rhs).ok(),
            )
        };

        // make sure we always do things in the same order
        if back.0 > forward.0 {
            std::mem::swap(&mut forward, &mut back);
        }

        match (forward, back) {
            ((_, _, _, None), (_, _, _, None)) => None,
            ((name, lhs, rhs, Some(rw)), (_, _, _, None))
            | ((_, _, _, None), (name, lhs, rhs, Some(rw))) => Some(Self {
                name: name.into(),
                lhs,
                rhs,
                ids: None,
                rewrites: vec![rw],
            }),
            ((_, lhs, rhs, Some(rw1)), (_, _, _, Some(rw2))) => Some(Self {
                // HACK: same as above
                name: format!("{} <=> {}", lhs, rhs).replace("\"", "").into(),
                lhs,
                rhs,
                ids: None,
                rewrites: if rw1.name == rw2.name {
                    vec![rw1]
                } else {
                    vec![rw1, rw2]
                },
            }),
        }
    }

    /// Assign a score to this Equality using heursitics mentioned in the Ruler paper
    /// (page 11, footnote 5).
    pub fn score(&self) -> impl Ord + Debug {
        L::score(&self.lhs, &self.rhs)
    }
}
