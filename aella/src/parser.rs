#![allow(clippy::upper_case_acronyms)]

use pest::prec_climber as pcl;
use pest::prec_climber::PrecClimber;
use pest_consume::{match_nodes, Error, Parser};
use std::{fs, path::Path};

use crate::lang;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "syntax.pest"]
pub struct AellaParser;

lazy_static::lazy_static! {
    static ref PRECCLIMBER: PrecClimber<Rule> = PrecClimber::new(
        vec![
            pcl::Operator::new(Rule::add, pcl::Assoc::Left),
            pcl::Operator::new(Rule::subtract, pcl::Assoc::Left),
            pcl::Operator::new(Rule::multiply, pcl::Assoc::Left),
            pcl::Operator::new(Rule::divide, pcl::Assoc::Left),
        ]
    );
}

impl AellaParser {
    pub fn parse_file(path: &Path) -> lang::Command {
        let content = fs::read(path).unwrap();
        let string_content = std::str::from_utf8(&content).unwrap();
        let inputs = match AellaParser::parse(Rule::file, &string_content) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };

        let input = inputs.single().unwrap();

        AellaParser::file(input).unwrap()
    }
}

#[pest_consume::parser]
impl AellaParser {
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    fn int(input: Node) -> Result<lang::NumExpr> {
        let i = input.as_str().parse().map_err(|e| input.error(e))?;
        Ok(lang::NumExpr::Num(i))
    }

    fn var(input: Node) -> Result<String> {
        Ok(input.as_str().into())
    }

    fn atom(input: Node) -> Result<lang::NumExpr> {
        Ok(match_nodes!(input.into_children();
                [int(i)] => i,
                [var(v)] => lang::NumExpr::Id(v)
        ))
    }

    #[prec_climb(atom, PRECCLIMBER)]
    fn num_expr(l: lang::NumExpr, op: Node, r: lang::NumExpr) -> Result<lang::NumExpr> {
        match op.as_rule() {
            Rule::add => Ok(lang::NumExpr::Plus(Box::new(l), Box::new(r))),
            Rule::subtract => Ok(lang::NumExpr::Sub(Box::new(l), Box::new(r))),
            Rule::multiply => Ok(lang::NumExpr::Times(Box::new(l), Box::new(r))),
            Rule::divide => Ok(lang::NumExpr::Div(Box::new(l), Box::new(r))),
            _ => Err(op.error(format!("Rule {:?} isn't an operator", r)))?,
        }
    }

    fn num_cmp(input: Node) -> Result<lang::BoolExpr> {
        Ok(match_nodes!(input.into_children();
            [num_expr(e1), cmp_binop(op), num_expr(e2)] => {
                match op {
                    "==" => lang::BoolExpr::Eq(e1, e2),
                    "!=" => lang::BoolExpr::Not(Box::new(lang::BoolExpr::Eq(e1, e2))),
                    "<" => lang::BoolExpr::Lt(e1, e2),
                    x => panic!("Unknown op: {:?}", x)
                }
            }
        ))
    }

    fn cmp_binop(input: Node) -> Result<&str> {
        Ok(input.as_str())
    }

    fn bool_expr(input: Node) -> Result<lang::BoolExpr> {
        Ok(match_nodes!(input.into_children();
                [num_cmp(b)] => b
        ))
    }

    fn assign(input: Node) -> Result<lang::Command> {
        Ok(match_nodes!(input.into_children();
                [var(x), num_expr(n)] => lang::Command::Assign(x, n)
        ))
    }

    fn if_cmd(input: Node) -> Result<lang::Command> {
        Ok(match_nodes!(input.into_children();
            [bool_expr(b), seq(tb), seq(fb)] => {
                lang::Command::If(b, Box::new(tb), Box::new(fb))
            }
        ))
    }

    fn while_cmd(input: Node) -> Result<lang::Command> {
        Ok(match_nodes!(input.into_children();
            [bool_expr(b), seq(body)] => {
                lang::Command::While(b, Box::new(body))
            }
        ))
    }

    fn asm_expr(input: Node) -> Result<lang::NumExpr> {
        Ok(match_nodes!(input.into_children();
                [num_expr(n)] => n,
                [var(v)] => lang::NumExpr::Id(v)
        ))
    }

    fn asm_instr(input: Node) -> Result<&str> {
        Ok(input.as_str())
    }

    fn asm_ops(input: Node) -> Result<lang::Command> {
        Ok(match_nodes!(input.into_children();
            [asm_instr(op), var(v), asm_expr(e)..] => {
                lang::Command::asm_from_name(op, v, e.collect::<Vec<_>>())
            }
        ))
    }

    fn cmd(input: Node) -> Result<lang::Command> {
        Ok(match_nodes!(input.into_children();
            [assign(a)] => a,
            [if_cmd(ifc)] => ifc,
            [while_cmd(whilec)] => whilec,
            [asm_ops(a)] => a
        ))
    }

    fn seq(input: Node) -> Result<lang::Command> {
        Ok(match_nodes!(input.into_children();
                [cmd(c1)..] => lang::Command::Seq(c1.collect())
        ))
    }

    fn file(input: Node) -> Result<lang::Command> {
        Ok(match_nodes!(input.into_children();
                [seq(s), EOI(_)] => s
        ))
    }
}
