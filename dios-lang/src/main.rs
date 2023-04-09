mod cost;
mod desugar;
mod error;
mod fuzz;
mod handwritten;
mod lang;
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

/// Read a `synthesis::DiosConfig` from a path (represented as a `&str`).
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

    /// output dir
    #[argh(option, from_str_fn(read_path))]
    output_dir: Option<PathBuf>,

    /// cost fun
    #[argh(switch)]
    alt_cost: bool,
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

/// Synthesize a new ruleset using `Ruler`.
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

/// Run the entire phased eqsat compilation process on a Dios program.
///  - this first calls the existing Dios code to generate an input program
///  - once we have an input program, we construct and call a `comp-gen` compiler.
fn compile(opts: CompileOpts) -> Res<()> {
    log::debug!("{opts:#?}");

    let output_dir = if let Some(path) = opts.output_dir {
        path
    } else {
        PathBuf::from(format!("{}-out", opts.input.as_str()))
    };

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
    // log::debug!("input: {}", prog.pretty(80));

    let mut compiler: comp_gen::Compiler<lang::VecLang, (), _> =
        comp_gen::Compiler::with_cost_fn(if opts.alt_cost {
            cost::VecCostFn::alternative()
        } else {
            cost::VecCostFn::original()
        });

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

    // add litvec rules
    compiler
        .add_rules(
            handwritten::build_litvec_rule(opts.vector_width).into_iter(),
        )
        // .with_filter(|cm| cm.cd > 0.0)
        // .add_cutoff_phase("test", |cd, _ca| cd > 0.0)
        .output_rule_distribution("rule_distribution.csv", |x| x);

    // load configuration
    if let Some(config) = &opts.config {
        compiler.with_config(config);
    }

    // compiler.with_explanations();
    let (cost, prog, mut _eg) = compiler.compile(prog);
    // let mut expl = eg.explain_existance(&prog);
    // log::debug!("you exist bc:\n{}", expl.get_string());
    // log::debug!("you exist bc:\n{}", expl.get_flat_string());
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
    let _ = env_logger::builder().try_init();

    let args: Cmdline = argh::from_env();

    match args.nested {
        Commands::Synth(opts) => synth(opts),
        Commands::Compile(opts) => compile(opts),
    }
}
