use comp_gen::{
    ruler::egg::{self, define_language, Id},
    FromPattern, ToRecExpr,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum NumExpr {
    Num(i64),
    Id(String),
    #[serde(alias = "+")]
    Plus(Box<Self>, Box<Self>),
    #[serde(alias = "-")]
    Sub(Box<Self>, Box<Self>),
    #[serde(alias = "*")]
    Times(Box<Self>, Box<Self>),
    #[serde(alias = "/")]
    Div(Box<Self>, Box<Self>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum BoolExpr {
    Bool(bool),
    #[serde(alias = "==")]
    Eq(NumExpr, NumExpr),
    #[serde(alias = "<")]
    Lt(NumExpr, NumExpr),
    #[serde(alias = "!")]
    Not(Box<Self>),
    #[serde(alias = "&&")]
    And(Box<Self>, Box<Self>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Command {
    Seq(Vec<Self>),
    #[serde(alias = ":=")]
    Assign(String, NumExpr),
    If(BoolExpr, Box<Self>, Box<Self>),
    While(BoolExpr, Box<Self>),

    // assembly instructions
    Mov(String, NumExpr),
    Add(String, NumExpr, NumExpr),
    Sub(String, NumExpr, NumExpr),
    // Multiplication
    Smull(String, NumExpr, NumExpr),
    Sdiv(String, NumExpr, NumExpr),
}

impl Command {
    pub fn asm_from_name(name: &str, var: String, exprs: Vec<NumExpr>) -> Self {
        match name {
            "@mov" => Self::Mov(var, exprs[0].clone()),
            "@add" => Self::Add(var, exprs[0].clone(), exprs[1].clone()),
            "@sub" => Self::Sub(var, exprs[0].clone(), exprs[1].clone()),
            "@smull" => Self::Smull(var, exprs[0].clone(), exprs[1].clone()),
            "@sdiv" => Self::Sdiv(var, exprs[0].clone(), exprs[1].clone()),
            x => panic!("Unknown op: {}", x),
        }
    }
}

define_language! {
    #[derive(Serialize, Deserialize)]
    pub enum Aella {
    "+" = Plus([Id; 2]),
    "-" = Sub([Id; 2]),
    "*" = Times([Id; 2]),
    "/" = Div([Id; 2]),
    "==" = Eq([Id; 2]),
    "<" = Lt([Id; 2]),
    "!" = Not([Id; 1]),
    "&&" = And([Id; 2]),
    "seq" = Seq([Id; 2]),
    ":=" = Assign([Id; 2]),
    "while" = While([Id; 2]),
    "mov" = AsmMov([Id; 2]),
    "add" = AsmAdd([Id; 3]),
    "sub" = AsmSub([Id; 3]),
    "smull" = AsmSmull([Id; 3]),
    "sdiv" = AsmSdiv([Id; 3]),
    Num(i64),
    Var(egg::Symbol),
    }
}

impl ToRecExpr<Aella> for Command {
    fn to_recexpr(&self, expr: &mut egg::RecExpr<Aella>) -> Id {
        match &self {
            Command::Seq(items) => {
                if items.len() == 1 {
                    todo!()
                } else if items.len() > 1 {
                    let last = items[items.len() - 1].clone();
                    let last_id = last.to_recexpr(expr);
                    items[..items.len() - 1].into_iter().rfold(
                        last_id,
                        |acc, it| {
                            let next_id = it.to_recexpr(expr);
                            expr.add(Aella::Seq([next_id, acc]))
                        },
                    )
                } else {
                    unreachable!()
                }
            }
            Command::Assign(lvar, val) => {
                let lvar_id = expr.add(Aella::Var(lvar.into()));
                let val_id = val.to_recexpr(expr);
                expr.add(Aella::Assign([lvar_id, val_id]))
            }
            Command::If(_, _, _) => todo!(),
            Command::While(cond, body) => {
                let cond_id = cond.to_recexpr(expr);
                let body_id = body.to_recexpr(expr);
                expr.add(Aella::While([cond_id, body_id]))
            }
            Command::Mov(rd, src) => {
                let rd_id = expr.add(Aella::Var(rd.into()));
                let src_id = src.to_recexpr(expr);
                expr.add(Aella::AsmMov([rd_id, src_id]))
            }
            Command::Add(rd, rn, rm) => {
                let rd_id = expr.add(Aella::Var(rd.into()));
                let rn_id = rn.to_recexpr(expr);
                let rm_id = rm.to_recexpr(expr);
                expr.add(Aella::AsmAdd([rd_id, rn_id, rm_id]))
            }
            Command::Sub(rd, rn, rm) => {
                let rd_id = expr.add(Aella::Var(rd.into()));
                let rn_id = rn.to_recexpr(expr);
                let rm_id = rm.to_recexpr(expr);
                expr.add(Aella::AsmSub([rd_id, rn_id, rm_id]))
            }
            Command::Smull(rd, rn, rm) => {
                let rd_id = expr.add(Aella::Var(rd.into()));
                let rn_id = rn.to_recexpr(expr);
                let rm_id = rm.to_recexpr(expr);
                expr.add(Aella::AsmSmull([rd_id, rn_id, rm_id]))
            }
            Command::Sdiv(rd, rn, rm) => {
                let rd_id = expr.add(Aella::Var(rd.into()));
                let rn_id = rn.to_recexpr(expr);
                let rm_id = rm.to_recexpr(expr);
                expr.add(Aella::AsmSdiv([rd_id, rn_id, rm_id]))
            }
        }
    }
}

impl ToRecExpr<Aella> for NumExpr {
    fn to_recexpr(&self, expr: &mut egg::RecExpr<Aella>) -> Id {
        match self {
            NumExpr::Num(n) => expr.add(Aella::Num(*n)),
            NumExpr::Id(x) => expr.add(Aella::Var(x.into())),
            NumExpr::Plus(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(Aella::Plus([left_id, right_id]))
            }
            NumExpr::Sub(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(Aella::Sub([left_id, right_id]))
            }
            NumExpr::Times(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(Aella::Times([left_id, right_id]))
            }
            NumExpr::Div(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(Aella::Div([left_id, right_id]))
            }
        }
    }
}

impl ToRecExpr<Aella> for BoolExpr {
    fn to_recexpr(&self, expr: &mut egg::RecExpr<Aella>) -> Id {
        match self {
            BoolExpr::Bool(_) => todo!(),
            BoolExpr::Eq(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(Aella::Eq([left_id, right_id]))
            }
            BoolExpr::Lt(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(Aella::Lt([left_id, right_id]))
            }
            BoolExpr::Not(inner) => {
                let inner_id = inner.to_recexpr(expr);
                expr.add(Aella::Not([inner_id]))
            }
            BoolExpr::And(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(Aella::And([left_id, right_id]))
            }
        }
    }
}

impl FromPattern for Aella {
    fn from_pattern(pat: &egg::PatternAst<Self>) -> egg::RecExpr<Self> {
        pat.as_ref()
            .iter()
            .map(|node| match node {
                egg::ENodeOrVar::Var(v) => Aella::Var(v.to_string().into()),
                egg::ENodeOrVar::ENode(node) => node.clone(),
            })
            .collect::<Vec<_>>()
            .into()
    }
}
