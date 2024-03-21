mod alpha_rename;
mod cost;
mod desugar;
mod error;
mod fuzz;
mod handwritten;
mod lang;
mod letify;
mod library_learning;
mod rewriteconcats;
mod smt;
mod stringconversion;
mod synthesis;

use crate::desugar::Desugar;
use anyhow::Context;
use argh::FromArgs;
use babble::{
    ast_node::{self, combine_exprs, AstNode, Expr},
    co_occurrence::COBuilder,
    learn::LearnedLibrary,
    sexp::Sexp,
};
use egg::LanguageMapper;
pub use error::Res;
use log;
use std::{fs, io::Write, path::PathBuf, process, time::Instant};

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

    /// dios binary
    #[allow(unused)]
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
    #[argh(option)]
    costfn: String,
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
    let exprs: lang::RecAst = babble::sexp::Program::parse(&concats?)
        .expect("Failed to parse program")
        .0
        .into_iter()
        .map(|x: Sexp<'_>| x.try_into().expect("Not valid list of expressions"))
        .map(|x: Expr<lang::VecOp>| x.into())
        .next()
        .expect("Found empty program");

    let prog: egg::RecExpr<lang::FlatAst> = exprs.into();

    log::info!("{}", prog.pretty(80));

    let mut compiler: comp_gen::Compiler<lang::FlatAst, (), _> =
        comp_gen::Compiler::with_cost_fn(match opts.costfn.as_str() {
            "alternative" => cost::VecCostFn::alternative(),
            "dios" => cost::VecCostFn::dios(),
            "accurate" => cost::VecCostFn::accurate(),
            _ => panic!("Not a valid cost function."),
        });

    // // add rules to compiler
    compiler.with_init_node(lang::FlatAst::new_const(lang::Value::Int(0)));

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
        .output_rule_distribution("rule_distribution.csv", |x| x);

    // load configuration
    if let Some(config) = &opts.config {
        compiler.with_config(config);
    }

    // compiler.with_explanations();
    // let (cost, prog, mut eg) = compiler.compile(prog);
    let eqsat_res = compiler.compile(prog);
    log::info!("cost: {}", eqsat_res.cost);

    let astnode_egraph =
        egg::SimpleLanguageMapper::default().map_egraph(eqsat_res.egraph);

    // first we have to run the co-occurence analysis
    log::info!("Running co-occurence analysis...");
    let co_time = Instant::now();
    let co_ext: COBuilder<lang::VecOp, ()> =
        COBuilder::new(&astnode_egraph, &eqsat_res.roots);
    let co_occurs = co_ext.run();
    log::info!("Finished in {}ms", co_time.elapsed().as_millis());

    log::info!("Running anti-unification... ");
    let au_time = Instant::now();
    let learned_lib =
        LearnedLibrary::new(&astnode_egraph, false, None, co_occurs);
    log::info!(
        "Found {} patterns in {}ms",
        learned_lib.size(),
        au_time.elapsed().as_millis()
    );

    let _all_libs: Vec<_> =
        learned_lib.libs().inspect(|l| log::info!("{l}")).collect();

    // // test reading in as the library learning thing
    // // I guess that at some point, I'm going to have to implement this so that the
    // // egraph operates over a different type of term. but we'll worry about that later
    // // let's just see if the parsing works the way that I think that it might.
    // {
    //     // let str_prog = prog.pretty(80);
    //     // let program: Vec<Expr<VecOp>> = Program::parse(&str_prog)
    //     //     .expect("Failed to parse")
    //     //     .0
    //     //     .into_iter()
    //     //     .map(|x| x.try_into().unwrap())
    //     //     .collect();
    //     // println!("{program:#?}");
    // }

    // // // test the let intro rewrite rule
    // // {
    // //     println!("test:\n{}", prog.clone().letify().pretty(80));
    // // }

    // write to spec.rkt
    let path = output_dir.join("res.rkt");
    let mut spec_file = fs::File::create(&path)?;
    log::debug!("writing to {path:?}");
    writeln!(spec_file, "{}", eqsat_res.prog)?;

    // call ./dios -w <vec_width> --egg --suppress-git -o <dir>/kernel.c <dir>
    // this generates the kernel.c file
    let output = process::Command::new(opts.dios_bin)
        .arg("-w")
        .arg(opts.vector_width.to_string())
        .arg("--egg")
        .arg("--suppress-git")
        .arg("--dump-intermediate")
        .arg("-o")
        .arg(output_dir.join("kernel.c"))
        .arg(output_dir)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::inherit())
        .output()?;
    println!("{}", String::from_utf8(output.stdout).unwrap());

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
