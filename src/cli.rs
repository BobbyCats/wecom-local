use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use crate::{auth, doctor, local_query, members, output, store_probe};

#[derive(Parser)]
#[command(
    name = "wecom-local",
    version = env!("CARGO_PKG_VERSION"),
    about = "Query locally visible WeCom Desktop data"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check or prepare local runtime authorization.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Check whether the local WeCom Desktop runtime can be reached.
    Doctor {
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Inspect local WeCom database file capabilities without reading row values.
    StoreProbe {
        /// Emit machine-readable JSON. Store probe currently only supports JSON output.
        #[arg(long)]
        json: bool,
    },
    /// List locally visible WeCom conversations.
    Conversations {
        /// Filter conversations by id or display name.
        #[arg(long)]
        query: Option<String>,
    },
    /// Read messages from a WeCom conversation id or display-name query.
    History {
        /// WeCom conversation id or display-name query, for example R:0000000000 or "Example Group".
        conversation: String,
        /// Maximum messages to return. Use 0 for all available message ids.
        #[arg(short = 'n', long, default_value = "50")]
        limit: usize,
        /// Message offset in the conversation message-id list.
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Output format.
        #[arg(short = 'f', long, value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Search decoded messages in one WeCom conversation.
    Search {
        /// Keyword to search for in decoded message text and sender display fields.
        query: String,
        /// WeCom conversation id or display-name query to search in.
        #[arg(long = "in")]
        conversation: String,
        /// Maximum matches to return. Use 0 for all matches in the scan window.
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
        /// Maximum recent messages to scan. Use 0 for all available message ids.
        #[arg(long, default_value = "1000")]
        max_scan: usize,
        /// Emit JSON. Search currently only supports JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Summarize decoded messages in one WeCom conversation.
    Stats {
        /// WeCom conversation id or display-name query to summarize.
        conversation: String,
        /// Maximum recent messages to scan. Use 0 for all available message ids.
        #[arg(long, default_value = "1000")]
        max_scan: usize,
        /// Include aggregate member participation counts without returning the member list.
        #[arg(long)]
        include_members: bool,
        /// Emit JSON. Stats currently only supports JSON output.
        #[arg(long)]
        json: bool,
    },
    /// List members in a WeCom conversation.
    Members {
        /// WeCom conversation id or display-name query to inspect.
        conversation: String,
        /// Include sensitive locally visible profile fields such as accounts, email, phone, and external ids.
        #[arg(long)]
        full: bool,
        /// Output format. Members currently supports JSON output.
        #[arg(short = 'f', long, value_enum, default_value = "json")]
        format: JsonOutputFormat,
    },
    /// Export messages from a WeCom conversation id or display-name query to a file.
    Export {
        /// WeCom conversation id or display-name query, for example R:0000000000 or "Example Group".
        conversation: String,
        /// Maximum messages to export. Use 0 for all available message ids.
        #[arg(short = 'n', long, default_value = "0")]
        limit: usize,
        /// Message offset in the conversation message-id list.
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Output format.
        #[arg(short = 'f', long, value_enum, default_value = "markdown")]
        format: OutputFormat,
        /// Output file path.
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum AuthCommand {
    /// Check whether sudo runtime authorization is currently ready.
    Status {
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Prompt through sudo/PAM to warm the local authorization timestamp.
    Prepare {
        /// Emit machine-readable JSON after preparation.
        #[arg(long)]
        json: bool,
        /// Keep the sudo timestamp refreshed while this command remains running.
        #[arg(long, default_value = "0")]
        keepalive_minutes: u16,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Json,
    Markdown,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum JsonOutputFormat {
    Json,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Auth { command } => match command {
            AuthCommand::Status { json } => {
                let report = auth::status();
                if json {
                    output::print_json(&serde_json::to_value(report)?)
                } else {
                    output::print_auth_status(&report)
                }
            }
            AuthCommand::Prepare {
                json,
                keepalive_minutes,
            } => {
                let report = auth::prepare(keepalive_minutes)?;
                if json {
                    output::print_json(&serde_json::to_value(report)?)
                } else {
                    output::print_auth_prepare(&report)
                }
            }
        },
        Commands::Doctor { json } => {
            let report = doctor::run();
            output::print_doctor_report(&report, json)
        }
        Commands::StoreProbe { json: _ } => {
            let report = store_probe::run();
            output::print_json(&serde_json::to_value(report)?)
        }
        Commands::Conversations { query } => {
            let payload = local_query::discover_conversations(query.as_deref())?;
            output::print_json(&payload)
        }
        Commands::History {
            conversation,
            limit,
            offset,
            format,
        } => {
            let payload = local_query::read_history(&conversation, limit, offset)?;
            print_payload(&payload, format)
        }
        Commands::Search {
            query,
            conversation,
            limit,
            max_scan,
            json: _,
        } => {
            let payload = local_query::search_messages(&conversation, &query, limit, max_scan)?;
            output::print_json(&payload)
        }
        Commands::Stats {
            conversation,
            max_scan,
            include_members,
            json: _,
        } => {
            let payload =
                local_query::conversation_stats(&conversation, max_scan, include_members)?;
            output::print_json(&payload)
        }
        Commands::Members {
            conversation,
            full,
            format: _,
        } => {
            let scope = if full {
                members::MemberDetailScope::Full
            } else {
                members::MemberDetailScope::Basic
            };
            let payload = local_query::list_members(&conversation, scope)?;
            output::print_json(&payload)
        }
        Commands::Export {
            conversation,
            limit,
            offset,
            format,
            output: path,
        } => {
            let payload = local_query::read_history(&conversation, limit, offset)?;
            output::write_payload(&payload, output_format(format), &path)
        }
    }
}

fn print_payload(payload: &serde_json::Value, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => output::print_json(payload),
        OutputFormat::Markdown => {
            println!("{}", output::to_markdown(payload));
            Ok(())
        }
    }
}

fn output_format(format: OutputFormat) -> output::Format {
    match format {
        OutputFormat::Json => output::Format::Json,
        OutputFormat::Markdown => output::Format::Markdown,
    }
}
