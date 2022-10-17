use std::{collections::HashMap, fmt::Display};

pub fn walk_recexpr_help<F, L: egg::Language>(
    expr: &egg::RecExpr<L>,
    id: egg::Id,
    action: F,
) -> F
where
    F: FnMut(&L),
{
    let mut f = action;
    f(&expr[id]);
    for c in expr[id].children() {
        let newf = walk_recexpr_help(expr, *c, f);
        f = newf;
    }
    f
}

pub fn walk_recexpr<F, L: egg::Language>(expr: &egg::RecExpr<L>, action: F)
where
    F: FnMut(&L),
{
    walk_recexpr_help(expr, (expr.as_ref().len() - 1).into(), action);
}

pub fn fold_recexpr<F, L: egg::Language, T>(
    expr: &egg::RecExpr<L>,
    init: T,
    mut action: F,
) -> T
where
    F: FnMut(T, &L) -> T,
    T: Clone,
{
    let mut acc = init;
    walk_recexpr(expr, |l| acc = action(acc.clone(), l));
    acc
}

pub fn root<L: egg::Language>(expr: &egg::RecExpr<L>) -> &L {
    &expr.as_ref()[expr.as_ref().len() - 1]
}

pub trait LanguageHelpers<L: egg::Language + Display> {
    fn build_recexpr_w_parent<F>(
        &self,
        root_id: egg::Id,
        get_node: &F,
    ) -> egg::RecExpr<Self>
    where
        F: Fn(egg::Id, egg::Id) -> Self,
        Self: Sized;
}

impl<L: egg::Language + Display> LanguageHelpers<L> for L {
    fn build_recexpr_w_parent<F>(
        &self,
        root_id: egg::Id,
        get_node: &F,
    ) -> egg::RecExpr<Self>
    where
        F: Fn(egg::Id, egg::Id) -> Self,
    {
        let children = self.children();
        // base case
        if children.is_empty() {
            return egg::RecExpr::from(vec![self.clone()]);
        }

        // eprintln!("{self:?}");

        // inductive case
        let map: HashMap<_, _> = self
            .children()
            .iter()
            .map(|&child_id| {
                let next_node = get_node(root_id, child_id);
                // eprintln!("  ({root_id}, {child_id}) -> ");
                let v = next_node.build_recexpr_w_parent(child_id, get_node);
                // eprintln!("    {}", v.pretty(80));
                (child_id, v)
            })
            .collect();

        self.join_recexprs(|id| map[&id].clone())
    }
}
