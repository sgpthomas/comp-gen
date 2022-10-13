mod cost;
mod desugar;
mod error;
mod lang;
#[allow(unused)]
mod recexpr_helpers;
mod rewriteconcats;
mod stringconversion;
mod synthesis;

use crate::desugar::Desugar;
use argh::FromArgs;
use comp_gen::ruler;
pub use error::Res;
use std::{fs, path::PathBuf, process};

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
    #[argh(option)]
    config: Option<PathBuf>,

    /// path to a ruler synthesis configuration file
    #[argh(option)]
    ruler: Option<PathBuf>,
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

    /// dios example params
    #[argh(option, from_str_fn(read_path))]
    dios_params: PathBuf,

    /// vector width
    #[argh(option)]
    vector_width: usize,

    /// ruleset
    #[argh(option, from_str_fn(read_path))]
    rules: PathBuf,

    /// config
    #[argh(option, from_str_fn(read_config))]
    config: Option<comp_gen::config::CompilerConfiguration>,
}

fn read_path(path: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(path))
}

fn read_config(
    path: &str,
) -> Result<comp_gen::config::CompilerConfiguration, String> {
    let file = fs::File::open(path).map_err(|e| format!("{e}"))?;
    let compile_config =
        serde_json::from_reader(file).map_err(|e| format!("{e}"))?;
    Ok(compile_config)
}

fn synth(synth_opts: SynthOpts) -> Res<()> {
    let args = synth_opts
        .ruler
        .as_ref()
        .map(|path| ruler::SynthParams::from_path(&path))
        .unwrap_or_else(|| Ok(ruler::SynthParams::default()))?;

    let report = synthesis::run(args, synth_opts.clone())?;
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

    log::debug!("input: {}", prog.pretty(80));

    let mut compiler: comp_gen::Compiler<lang::VecLang, (), _> =
        comp_gen::Compiler::with_cost_fn(cost::VecCostFn::default());

    // add rules to compiler
    compiler
        .with_init_node(lang::VecLang::Const(lang::Value::Int(0)))
        .add_processed_external_rules(&opts.rules, |p| {
            p.desugar(opts.vector_width)
        })
        // .with_filter(|cm| cm.cd > 0.0)
        // .add_cutoff_phase("test", |cd, _ca| cd > 0.0)
        .output_rule_distribution("rule_distribution.csv", |x| x);

    // load configuration
    if let Some(config) = &opts.config {
        compiler.with_config(config);
    }

    let (cost, prog, _eg) = compiler.compile(&prog);
    if let Some(cost) = cost {
        log::info!("cost: {cost}");
    }
    // eg.dot().to_png("test.png").expect("failed to create image");
    log::info!("{}", prog.pretty(80));

    // cleanup dir
    fs::remove_dir_all(output_dir)?;

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
