mod cost;
pub mod interpret;
mod lang;
mod parser;
mod synthesis;

use argh::FromArgs;
use chain_cmp::chmp;
use comp_gen::{ruler, ToRecExpr};
use egg::RecExpr;
use std::path::{Path, PathBuf};

use crate::lang::Command;

/// Options
#[derive(FromArgs)]
struct Cmdline {
    #[argh(subcommand)]
    nested: Commands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Commands {
    /// synthesize a new ruleset
    Synth(SynthOpts),
    /// compile an input program using a ruleset.
    Compile(CompileOpts),
}

#[derive(Clone, FromArgs)]
#[argh(subcommand, name = "synth")]
/// Synth options.
pub struct SynthOpts {
    #[argh(positional)]
    output: String,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "compile")]
/// Compile options.
struct CompileOpts {
    /// input file.
    #[argh(positional)]
    input: String,

    /// ruleset.
    #[argh(option, from_str_fn(read_path))]
    rules: PathBuf,
}

fn read_path(path: &str) -> Result<PathBuf, String> {
    Ok(Path::new(path).into())
}

fn synth(synth_opts: SynthOpts) {
    let args: ruler::SynthParams = ruler::SynthParams {
        variables: 4,
        iters: 2,
        abs_timeout: 240,
        do_final_run: true,
        eqsat_node_limit: 2_000_000,
        eqsat_iter_limit: 2,
        eqsat_time_limit: 10,
        ..ruler::SynthParams::default()
    };
    let report = synthesis::run(args, synth_opts.clone());
    let file = std::fs::File::create(&synth_opts.output)
        .unwrap_or_else(|_| panic!("Failed to open '{}'", &synth_opts.output));
    serde_json::to_writer_pretty(file, &report).expect("failed to write json");
}

fn compile(opts: CompileOpts) {
    // read in prog and convert it to a egg rec expression
    let path = Path::new(&opts.input);
    let cmd: Command = parser::AellaParser::parse_file(path);
    let mut expr = RecExpr::default();
    cmd.to_recexpr(&mut expr);

    println!("{}", expr.pretty(80));

    let compiler: comp_gen::Compiler<lang::Aella, (), _> =
        comp_gen::Compiler::with_cost_fn(cost::AellaCost::default())
            .with_timeout(240)
            .with_node_limit(1_000_000)
            .with_init_node(lang::Aella::Num(0))
            .add_external_rules(&opts.rules)
            .output_rule_distribution("rule_distribution.csv", |x| x)
            .dry_run()
            .dump_rules()
            .add_cutoff_phase("pre compile", |cd, ca| cd.abs() < 1.0 && ca.abs() < 0.5)
            .add_cutoff_phase("compile", |cd, ca| {
                chmp!(1.0 < cd.abs() < 3.0) && chmp!(0.5 < ca.abs() < 1.5)
            })
            .add_cutoff_phase("optimization", |cd, ca| 3.0 < cd.abs() && 1.5 < ca.abs());

    let (cost, prog, _eg) = compiler.compile(&expr);
    if let Some(cost) = cost {
        println!("cost: {cost}");
    }
    // eg.dot().to_png("test.png").expect("failed to create image");
    println!("{}", prog.pretty(80));
}

fn main() {
    let _ = env_logger::builder().try_init();

    let args: Cmdline = argh::from_env();

    match args.nested {
        Commands::Synth(opts) => synth(opts),
        Commands::Compile(opts) => compile(opts),
    }
}
