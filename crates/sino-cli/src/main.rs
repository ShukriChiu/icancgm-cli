use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use serde_json::Value;
use sino_core::credentials::{Credential, CredentialManager};
use sino_core::sources::{self, AuthType};
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
    /// 管理数据源凭证
    Auth(AuthCommand),
    User(UserCommand),
    Cgm(CgmCommand),
    Daily(DateLookupArgs),
    Event(EventCommand),
}

// ── Auth ─────────────────────────────────────────────────────────

#[derive(Debug, Args)]
struct AuthCommand {
    #[command(subcommand)]
    command: AuthSubcommand,
}

#[derive(Debug, Subcommand)]
enum AuthSubcommand {
    /// 添加或更新数据源凭证
    Add(AuthAddArgs),
    /// 列出已配置的数据源
    List,
    /// 移除数据源凭证
    Remove(AuthRemoveArgs),
    /// 查看数据源凭证获取指南
    Guide(AuthGuideArgs),
}

#[derive(Debug, Args)]
struct AuthAddArgs {
    /// 数据源名称 (sinocare, oura, dexcom, libre)
    source: String,
    #[arg(long, help = "用户 ID（sinocare）")]
    user_id: Option<String>,
    #[arg(long, help = "访问令牌（oura）")]
    token: Option<String>,
    #[arg(long, help = "刷新令牌（oura，可选）")]
    refresh_token: Option<String>,
    #[arg(long, help = "用户名（dexcom, libre）")]
    username: Option<String>,
    #[arg(long, help = "密码（dexcom, libre）")]
    password: Option<String>,
}

#[derive(Debug, Args)]
struct AuthRemoveArgs {
    /// 数据源名称
    source: String,
}

#[derive(Debug, Args)]
struct AuthGuideArgs {
    /// 数据源名称 (sinocare, oura, dexcom, libre, apple-health)
    source: String,
}

// ── Data commands ────────────────────────────────────────────────

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
    #[arg(long, help = "用户 ID（可省略，自动从凭证读取）")]
    user_id: Option<String>,
}

#[derive(Debug, Args)]
struct DateLookupArgs {
    #[arg(long, help = "用户 ID（可省略，自动从凭证读取）")]
    user_id: Option<String>,
    #[arg(long)]
    date: String,
}

#[derive(Debug, Args)]
struct DateRangeLookupArgs {
    #[arg(long, help = "用户 ID（可省略，自动从凭证读取）")]
    user_id: Option<String>,
    #[arg(long)]
    start_date: String,
    #[arg(long)]
    end_date: String,
}

#[derive(Debug, Args)]
struct EventLookupArgs {
    #[arg(long, help = "用户 ID（可省略，自动从凭证读取）")]
    user_id: Option<String>,
    #[arg(long)]
    event_id: String,
}

// ── Main ─────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Auth(cmd) => handle_auth(cmd),
        other => handle_data_command(other, cli.json, cli.pretty).await,
    }
}

// ── Auth handlers ────────────────────────────────────────────────

fn handle_auth(cmd: AuthCommand) -> Result<()> {
    match cmd.command {
        AuthSubcommand::Add(args) => auth_add(args),
        AuthSubcommand::List => auth_list(),
        AuthSubcommand::Remove(args) => auth_remove(args),
        AuthSubcommand::Guide(args) => auth_guide(args),
    }
}

fn auth_add(args: AuthAddArgs) -> Result<()> {
    let source_meta = sources::find_source(&args.source).ok_or_else(|| {
        let known = sources::known_source_names().join(", ");
        anyhow!(
            "未知的数据源: {}\n可用数据源: {known}\n运行 sino auth guide <source> 查看配置方法",
            args.source
        )
    })?;

    let credential = match source_meta.auth_type {
        AuthType::UserId => {
            let user_id = args.user_id.ok_or_else(|| {
                anyhow!(
                    "sinocare 需要 --user-id 参数\n\n{}",
                    source_meta.format_guide()
                )
            })?;
            Credential::UserId { user_id }
        }
        AuthType::OAuthToken => {
            let access_token = args.token.ok_or_else(|| {
                anyhow!(
                    "oura 需要 --token 参数\n\n{}",
                    source_meta.format_guide()
                )
            })?;
            Credential::OauthToken {
                access_token,
                refresh_token: args.refresh_token,
                expires_at: None,
            }
        }
        AuthType::Password => {
            let username = args.username.ok_or_else(|| {
                anyhow!(
                    "{} 需要 --username 参数\n\n{}",
                    args.source,
                    source_meta.format_guide()
                )
            })?;
            let password = args.password.ok_or_else(|| {
                anyhow!(
                    "{} 需要 --password 参数\n\n{}",
                    args.source,
                    source_meta.format_guide()
                )
            })?;
            Credential::Password { username, password }
        }
        AuthType::LocalExport => {
            println!("{}", source_meta.format_guide());
            println!("💡 {} 无需配置凭证", source_meta.display_name);
            return Ok(());
        }
    };

    let mut manager = CredentialManager::load()?;
    let old = manager.set(&args.source, credential);
    manager.save()?;

    match old {
        Some(old_cred) => {
            let new_cred = manager.get(&args.source).unwrap();
            eprintln!(
                "⚠ {} 凭证已更新（{} → {}）",
                source_meta.display_name,
                old_cred.display_summary(),
                new_cred.display_summary()
            );
        }
        None => {
            eprintln!("✓ {} 凭证已保存", source_meta.display_name);
        }
    }

    Ok(())
}

fn auth_list() -> Result<()> {
    let manager = CredentialManager::load()?;
    let configured = manager.list();

    eprintln!("数据源凭证状态:\n");

    let mut has_unconfigured = false;
    for source in sources::all_sources() {
        if source.auth_type == AuthType::LocalExport {
            continue;
        }
        match configured.get(source.name) {
            Some(cred) => {
                let status = if cred.is_expired() { "⚠" } else { "✓" };
                eprintln!(
                    "  {status} {:<14} ({})",
                    source.display_name,
                    cred.display_summary()
                );
            }
            None => {
                has_unconfigured = true;
                eprintln!("  ✗ {:<14} (未配置)", source.display_name);
            }
        }
    }

    if has_unconfigured {
        eprintln!("\n💡 运行 sino auth guide <source> 查看配置方法");
    }

    Ok(())
}

fn auth_remove(args: AuthRemoveArgs) -> Result<()> {
    let mut manager = CredentialManager::load()?;
    match manager.remove(&args.source) {
        Some(_) => {
            manager.save()?;
            let display = sources::find_source(&args.source)
                .map(|s| s.display_name)
                .unwrap_or(&args.source);
            eprintln!("✓ {display} 凭证已移除");
        }
        None => {
            eprintln!("⚠ {} 没有已配置的凭证", args.source);
        }
    }
    Ok(())
}

fn auth_guide(args: AuthGuideArgs) -> Result<()> {
    let source = sources::find_source(&args.source).ok_or_else(|| {
        let known = sources::known_source_names().join(", ");
        anyhow!(
            "未知的数据源: {}\n可用数据源: {known}",
            args.source
        )
    })?;

    println!("{}", source.format_guide());
    Ok(())
}

// ── Data command handlers ────────────────────────────────────────

/// Resolve user_id: CLI arg > credential store > error with guide.
fn resolve_user_id(explicit: Option<String>) -> Result<String> {
    if let Some(id) = explicit {
        return Ok(id);
    }

    let manager = CredentialManager::load()?;
    if let Some(id) = manager.get_sinocare_user_id() {
        return Ok(id.to_string());
    }

    let guide = sources::find_source("sinocare")
        .map(|s| s.format_guide())
        .unwrap_or_default();
    Err(anyhow!(
        "缺少 --user-id 参数，且未配置 sinocare 凭证\n\n\
         请先运行 sino auth add sinocare --user-id <ID> 保存凭证，\n\
         或使用 --user-id 参数临时指定\n\n{guide}"
    ))
}

async fn handle_data_command(command: Command, json: bool, pretty: bool) -> Result<()> {
    let client = SinoClient::new(AppConfig::default())?;

    let value = match command {
        Command::Health => client.health().await?,
        Command::User(cmd) => match cmd.command {
            UserSubcommand::Info(args) => {
                let uid = resolve_user_id(args.user_id)?;
                client.user_info(&uid).await?
            }
        },
        Command::Cgm(cmd) => match cmd.command {
            CgmSubcommand::Day(args) => {
                let uid = resolve_user_id(args.user_id)?;
                client.cgm_day(&uid, &args.date).await?
            }
            CgmSubcommand::Range(args) => {
                let uid = resolve_user_id(args.user_id)?;
                client
                    .cgm_range(&uid, &args.start_date, &args.end_date)
                    .await?
            }
        },
        Command::Daily(args) => {
            let uid = resolve_user_id(args.user_id)?;
            client.daily(&uid, &args.date).await?
        }
        Command::Event(cmd) => match cmd.command {
            EventSubcommand::Get(args) => {
                let uid = resolve_user_id(args.user_id)?;
                client.event(&uid, &args.event_id).await?
            }
        },
        Command::Auth(_) => unreachable!(),
    };

    print_output(&value, json || !pretty)?;
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
