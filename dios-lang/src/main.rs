mod error;
mod synthesis;

use argh::FromArgs;
use comp_gen::ruler;
pub use error::Res;
use std::path::PathBuf;

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
}
fn synth(synth_opts: SynthOpts) -> Res<()> {
    let args: ruler::SynthParams = ruler::SynthParams {
        variables: 6,
        iters: 2,
        abs_timeout: 240,
        do_final_run: true,
        eqsat_node_limit: 2_000_000,
        eqsat_iter_limit: 2,
        eqsat_time_limit: 10,
        ..ruler::SynthParams::default()
    };
    let report = synthesis::run(args, synth_opts.clone())?;
    let file = std::fs::File::create(&synth_opts.output)
        .unwrap_or_else(|_| panic!("Failed to open '{}'", &synth_opts.output));
    serde_json::to_writer_pretty(file, &report).expect("failed to write json");
    Ok(())
}

fn main() -> Res<()> {
    let _ = env_logger::builder().try_init();

    let args: Cmdline = argh::from_env();

    match args.nested {
        Commands::Synth(opts) => synth(opts),
    }
}
