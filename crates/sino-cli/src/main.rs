use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use serde_json::Value;
use sino_core::{AppConfig, SinoClient};

#[derive(Debug, Parser)]
#[command(name = "sino", version, about = "SINO CGM CLI for production SCRM MCP")]
struct Cli {
    #[arg(long, global = true, help = "输出紧凑 JSON")]
    json: bool,
    #[arg(long, global = true, help = "输出格式化 JSON")]
    pretty: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Health,
    User(UserCommand),
    Cgm(CgmCommand),
    Daily(DateLookupArgs),
    Event(EventCommand),
}

#[derive(Debug, Args)]
struct UserCommand {
    #[command(subcommand)]
    command: UserSubcommand,
}

#[derive(Debug, Subcommand)]
enum UserSubcommand {
    Info(UserIdArgs),
}

#[derive(Debug, Args)]
struct CgmCommand {
    #[command(subcommand)]
    command: CgmSubcommand,
}

#[derive(Debug, Subcommand)]
enum CgmSubcommand {
    Day(DateLookupArgs),
    Range(DateRangeLookupArgs),
}

#[derive(Debug, Args)]
struct EventCommand {
    #[command(subcommand)]
    command: EventSubcommand,
}

#[derive(Debug, Subcommand)]
enum EventSubcommand {
    Get(EventLookupArgs),
}

#[derive(Debug, Args)]
struct UserIdArgs {
    #[arg(long)]
    user_id: String,
}

#[derive(Debug, Args)]
struct DateLookupArgs {
    #[arg(long)]
    user_id: String,
    #[arg(long)]
    date: String,
}

#[derive(Debug, Args)]
struct DateRangeLookupArgs {
    #[arg(long)]
    user_id: String,
    #[arg(long)]
    start_date: String,
    #[arg(long)]
    end_date: String,
}

#[derive(Debug, Args)]
struct EventLookupArgs {
    #[arg(long)]
    user_id: String,
    #[arg(long)]
    event_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = SinoClient::new(AppConfig::default())?;

    let value = match cli.command {
        Command::Health => client.health().await?,
        Command::User(cmd) => match cmd.command {
            UserSubcommand::Info(args) => client.user_info(&args.user_id).await?,
        },
        Command::Cgm(cmd) => match cmd.command {
            CgmSubcommand::Day(args) => client.cgm_day(&args.user_id, &args.date).await?,
            CgmSubcommand::Range(args) => {
                client
                    .cgm_range(&args.user_id, &args.start_date, &args.end_date)
                    .await?
            }
        },
        Command::Daily(args) => client.daily(&args.user_id, &args.date).await?,
        Command::Event(cmd) => match cmd.command {
            EventSubcommand::Get(args) => client.event(&args.user_id, &args.event_id).await?,
        },
    };

    print_output(&value, cli.json || !cli.pretty)?;
    Ok(())
}

fn print_output(value: &Value, compact: bool) -> Result<()> {
    let text = if compact {
        serde_json::to_string(value)?
    } else {
        serde_json::to_string_pretty(value)?
    };

    println!("{text}");
    Ok(())
}
