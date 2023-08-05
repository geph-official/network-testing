use clap::{Parser, Subcommand};

use crate::execute::Monitor;

mod execute;
mod install;

#[derive(Parser)]
#[clap(version = "1.0")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Execute(ExecuteArgs),
    Browse(BrowseArgs),
}

#[derive(Parser)]
struct BrowseArgs {}

#[derive(Parser)]
struct ExecuteArgs {
    #[clap(long)]
    all: bool,

    #[clap(long = "address")]
    addr: Option<String>,

    #[clap(long)]
    script_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Execute(args) => {
            let monitors = execute::list_active_monitors().await?;
            println!("hi: {:?}", monitors);
            let monitors: Vec<Monitor> = monitors
                .into_iter()
                .filter(|m| m.attributes.port.is_some())
                .collect();
            println!("monitors: {:?}", monitors);
            if args.all {
                for monitor in monitors {
                    execute::execute_script(
                        &monitor.attributes.url,
                        &monitor
                            .attributes
                            .port
                            .expect("missing monitor port, even after validation?!"),
                        &args.script_url,
                    )
                    .await?;
                }
            } else if let Some(addr) = args.addr.clone() {
            } else {
                eprintln!("invalid args!")
            }
        }
        Commands::Browse(_args) => {
            todo!()
        }
    }
    Ok(())
}
