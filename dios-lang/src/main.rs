mod cost;
mod desugar;
mod error;
mod fuzz;
mod lang;
#[allow(unused)]
mod recexpr_helpers;
mod rewriteconcats;
mod smt;
mod stringconversion;
mod synthesis;

use crate::desugar::Desugar;
use anyhow::Context;
use argh::FromArgs;
use comp_gen::ruler::egg;
pub use error::Res;
use log::info;
use std::{fs, io::Write, path::PathBuf, process};

/// Generate and run an automatically generated compiler
/// for the Diospyros vector language.
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
    /// compile an input program using a ruleset
    Compile(CompileOpts),
}

#[derive(Clone, FromArgs)]
#[argh(subcommand, name = "synth")]
/// Synth options.
pub struct SynthOpts {
    #[argh(positional)]
    output: String,

    /// path to a dios configuration file
    #[argh(option, from_str_fn(read_synth_config))]
    config: Option<synthesis::DiosConfig>,

    /// path to a chkpt file
    #[argh(option, from_str_fn(read_path))]
    checkpoint: Option<PathBuf>,
}

fn read_synth_config(path: &str) -> Result<synthesis::DiosConfig, String> {
    let config_file = fs::File::open(path)
        .context("open config path")
        .map_err(|e| e.to_string())?;
    let parsed: synthesis::DiosConfig = serde_json::from_reader(config_file)
        .context(format!("parse {path:?} as dios_config"))
        .map_err(|e| e.to_string())?;
    Ok(parsed)
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "compile")]
/// Compile options.
struct CompileOpts {
    /// input file
    #[argh(positional)]
    input: String,

    /// dios-example-gen binary
    #[argh(option, from_str_fn(read_path))]
    dios_example_bin: PathBuf,

    /// diosbinary
    #[argh(option, from_str_fn(read_path))]
    dios_bin: PathBuf,

    /// dios example params
    #[argh(option, from_str_fn(read_path))]
    dios_params: PathBuf,

    /// vector width
    #[argh(option)]
    vector_width: usize,

    /// ruleset
    #[argh(option, from_str_fn(read_path))]
    rules: PathBuf,

    /// pre-desugared
    #[argh(switch)]
    pre_desugared: bool,

    /// config
    #[argh(option, from_str_fn(read_compiler_config))]
    config: Option<comp_gen::config::CompilerConfiguration>,
}

fn read_path(path: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(path))
}

fn read_compiler_config(
    path: &str,
) -> Result<comp_gen::config::CompilerConfiguration, String> {
    let file = fs::File::open(path).map_err(|e| format!("{e}"))?;
    let compile_config =
        serde_json::from_reader(file).map_err(|e| format!("{e}"))?;
    Ok(compile_config)
}

fn synth(synth_opts: SynthOpts) -> Res<()> {
    let report = synthesis::run(
        synth_opts.config.unwrap_or_default(),
        synth_opts.checkpoint,
    )?;
    let file = std::fs::File::create(&synth_opts.output)
        .unwrap_or_else(|_| panic!("Failed to open '{}'", &synth_opts.output));
    serde_json::to_writer_pretty(file, &report).expect("failed to write json");
    Ok(())
}

fn compile(opts: CompileOpts) -> Res<()> {
    log::debug!("{opts:#?}");

    let output_dir = PathBuf::from(format!("{}-out", opts.input.as_str()));

    // generate the example with dios_example_gen
    process::Command::new(opts.dios_example_bin)
        .arg("-w")
        .arg(opts.vector_width.to_string())
        .arg("-b")
        .arg(&opts.input)
        .arg("-o")
        .arg(&output_dir)
        .arg("-p")
        .arg(opts.dios_params)
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .output()?;

    // read in program
    let prog_str = fs::read_to_string(output_dir.join("spec.rkt"))?;
    let converted: String = stringconversion::convert_string(&prog_str)?;

    // rewrite into concats of vecs
    let concats =
        rewriteconcats::list_to_concats(opts.vector_width, &converted);
    // finally parse into a rec expr
    let prog: egg::RecExpr<lang::VecLang> = concats?.parse()?;
    // debug!("input: {}", prog.pretty(80));

    let mut compiler: comp_gen::Compiler<lang::VecLang, (), _> =
        comp_gen::Compiler::with_cost_fn(cost::VecCostFn::default());

    // add rules to compiler
    compiler.with_init_node(lang::VecLang::Const(lang::Value::Int(0)));

    // add predesugared rules
    if opts.pre_desugared {
        compiler.add_external_rules(&opts.rules);
    } else {
        compiler.add_processed_external_rules(&opts.rules, |p| {
            p.desugar(opts.vector_width)
        });
    }

    compiler
        .add_rules(
            vec![
                    // egg::rewrite!("sqrt-1-inv"; "1" => "(sqrt 1)"),
                    // egg::rewrite!("sqrt-1-inv-rev"; "(sqrt 1)" => "1"),
            // egg::rewrite!("div-1"; "(/ ?a 1)" => "?a"),
            // egg::rewrite!("div-1-inv"; "?a" => "(/ ?a 1)"),
            // egg::rewrite!("neg-sgn"; "(neg (sgn ?a))" => "(sgn (neg ?a))"),
            // egg::rewrite!("neg-sgn-rev"; "(sgn (neg ?a))" => "(neg (sgn ?a))"),
                    // egg::rewrite!("sqrt-vec"; "(Vec (sqrt ?a))" => "(VecSqrt ?a)"),
                    // egg::rewrite!("sqrt-vec-rev"; "(VecSqrt ?a)" => "(Vec (sqrt ?a))"),
                    // egg::rewrite!("sgn-vec"; "(Vec (sgn ?a))" => "(VecSgn ?a)"),
                    // egg::rewrite!("sgn-vec-rev"; "(VecSgn ?a)" => "(Vec (sgn ?a))"),
                    // egg::rewrite!("div"; "(Vec (/ ?a ?b) (/ ?c ?d))" => "(VecDiv (Vec ?a ?c) (Vec ?b ?d))"),
                    // egg::rewrite!("div-rev"; "(VecDiv (Vec ?a ?c) (Vec ?b ?d))" => "(Vec (/ ?a ?b) (/ ?c ?d))"),

                    // egg::rewrite!("blah2l"; "(neg (* ?b ?a))" => "(* (neg ?b) ?a)"),
                    // egg::rewrite!("blah2"; "(* (neg ?b) ?a)" => "(neg (* ?b ?a))"),
                    // egg::rewrite!("blah"; "0" => "(+ 0 0)"),
                    // egg::rewrite!("vec-neg"; "(Vec (neg ?a) (neg ?b))" => "(VecNeg (Vec ?a ?b))"),
                    // egg::rewrite!("-+neg"; "(- ?a ?b)" => "(+ ?a (neg ?b))"),
                    // egg::rewrite!("-+neg-rev"; "(+ ?a (neg ?b))" => "(- ?a ?b)"),
                    // egg::rewrite!("vec-neg0-l"; "(Vec 0 (neg ?b))" => "(VecNeg (Vec 0 ?b))"),
                    // egg::rewrite!("vec-neg0-r"; "(Vec (neg ?a) 0)" => "(VecNeg (Vec ?a 0))"),
                    ]
            .into_iter(),
        )
        // .with_filter(|cm| cm.cd > 0.0)
        // .add_cutoff_phase("test", |cd, _ca| cd > 0.0)
        .output_rule_distribution("rule_distribution.csv", |x| x);

    // load configuration
    if let Some(config) = &opts.config {
        compiler.with_config(config);
    }

    let (cost, prog, _eg) = compiler.compile(&prog);
    info!("cost: {cost}");
    // eg.dot().to_png("test.png").expect("failed to create image");
    info!("{}", prog.pretty(80));

    // write to spec.rkt
    let mut spec_file = fs::File::create(output_dir.join("res.rkt"))?;
    log::debug!("writing to {:?}", spec_file);
    writeln!(spec_file, "{}", prog.pretty(80))?;

    // call ./dios -w <vec_width> --egg --suppress-git -o <dir>/kernel.c <dir>
    // this generates the kernel.c file
    process::Command::new(opts.dios_bin)
        .arg("-w")
        .arg(opts.vector_width.to_string())
        .arg("--egg")
        .arg("--suppress-git")
        .arg("-o")
        .arg(output_dir.join("kernel.c"))
        .arg(output_dir)
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .output()?;

    Ok(())
}

fn main() -> Res<()> {
    let _ = env_logger::builder().format_timestamp(None).try_init();

    let args: Cmdline = argh::from_env();

    match args.nested {
        Commands::Synth(opts) => synth(opts),
        Commands::Compile(opts) => compile(opts),
    }
}
