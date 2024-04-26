use clap::Parser;
use proc_exit::prelude::*;

#[derive(Parser)]
#[command(about, author, version)]
#[command(group = clap::ArgGroup::new("mode").multiple(false))]
struct Args {
    #[arg(short, long, group = "mode")]
    input: Option<std::path::PathBuf>,
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
    /// Sleep between commits
    #[arg(long)]
    sleep: Option<humantime::Duration>,

    #[arg(long, group = "mode")]
    schema: Option<std::path::PathBuf>,
}

fn main() {
    let result = run();
    proc_exit::exit(result);
}

fn run() -> proc_exit::ExitResult {
    env_logger::init();

    let args = Args::parse();
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    if let Some(input) = args.input.as_deref() {
        std::fs::create_dir_all(&output).with_code(proc_exit::Code::FAILURE)?;
        let mut dag = git_fixture::TodoList::load(input).with_code(proc_exit::bash::USAGE)?;
        dag.sleep = dag.sleep.or_else(|| args.sleep.map(|s| s.into()));
        dag.run(&output).with_code(proc_exit::Code::FAILURE)?;
    } else if let Some(_schema_path) = args.schema.as_deref() {
        #[cfg(feature = "schema")]
        {
            use std::io::Write;

            let schema = schemars::schema_for!(git_fixture::TodoList);
            let schema = serde_json::to_string_pretty(&schema).unwrap();
            if _schema_path == std::path::Path::new("-") {
                std::io::stdout()
                    .write_all(schema.as_bytes())
                    .with_code(proc_exit::Code::FAILURE)?;
            } else {
                std::fs::write(_schema_path, &schema).with_code(proc_exit::Code::FAILURE)?;
            }
        }
        #[cfg(not(feature = "schema"))]
        {
            return Err(eyre::eyre!("schema is unsupported")).with_code(proc_exit::Code::FAILURE);
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;
        Args::command().debug_assert();
    }
}
