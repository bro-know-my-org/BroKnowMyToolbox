use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub(crate) enum DiagnosticsCommand {
    /// Report the effective shared data directory.
    DataDir {
        /// Emit stable machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum FileCommand {
    /// List templates compiled into this release.
    Templates {
        /// Emit stable machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Create files from a versioned template definition.
    Create {
        /// JSON template definition to load.
        #[arg(long)]
        template: PathBuf,
        /// Directory that will contain generated files.
        #[arg(long)]
        destination: PathBuf,
        /// Template variable in NAME=VALUE form. May be repeated.
        #[arg(long = "var", value_parser = parse_variable)]
        variables: Vec<(String, String)>,
        /// Produce a generation plan without writing files.
        #[arg(long)]
        dry_run: bool,
        /// Allow planned files to overwrite existing files.
        #[arg(long)]
        force: bool,
        /// Emit stable machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConsentCommand {
    /// Persist an allowed capability decision for a built-in tool.
    Allow { tool_id: String, capability: String },
    /// Persist a denied capability decision for a built-in tool.
    Deny { tool_id: String, capability: String },
}

#[derive(Debug, Subcommand)]
pub(crate) enum SparkCommand {
    /// List deterministic report tools exposed by Spark Analyzer.
    Tools {
        #[arg(long)]
        json: bool,
    },
    /// Parse a local report and print its summary.
    Inspect {
        source: PathBuf,
        /// Treat the report as UTF-8 text rather than spark protobuf.
        #[arg(long)]
        text: bool,
        #[arg(long)]
        json: bool,
    },
    /// Run one deterministic Spark Analyzer report tool.
    Tool {
        source: PathBuf,
        tool: String,
        #[arg(long)]
        text: bool,
        /// Complete JSON object passed to the tool.
        #[arg(long, conflicts_with = "arguments")]
        args: Option<String>,
        /// Tool argument in KEY=VALUE form. May be repeated.
        #[arg(long = "arg")]
        arguments: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Run the evidence-driven Spark Analyzer AI agent.
    Analyze {
        source: PathBuf,
        #[arg(long)]
        text: bool,
        #[arg(long, default_value = "https://api.openai.com/v1")]
        base_url: String,
        #[arg(long, default_value = "gpt-4.1-mini")]
        model: String,
        #[arg(long, default_value_t = 0.2, value_parser = parse_temperature)]
        temperature: f32,
        #[arg(long, default_value_t = 12, value_parser = parse_max_rounds)]
        max_rounds: usize,
        #[arg(long)]
        json: bool,
    },
}

fn parse_max_rounds(value: &str) -> Result<usize, String> {
    let value = value
        .parse::<usize>()
        .map_err(|_| "max rounds must be an integer between 1 and 64".to_string())?;
    if (1..=64).contains(&value) {
        Ok(value)
    } else {
        Err("max rounds must be between 1 and 64".to_string())
    }
}

fn parse_temperature(value: &str) -> Result<f32, String> {
    let value = value
        .parse::<f32>()
        .map_err(|_| "temperature must be a number between 0 and 2".to_string())?;
    if value.is_finite() && (0.0..=2.0).contains(&value) {
        Ok(value)
    } else {
        Err("temperature must be between 0 and 2".to_string())
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Inspect or change persisted product consent decisions.
    Consent {
        #[command(subcommand)]
        command: ConsentCommand,
    },
    /// Analyze spark profiler reports.
    Spark {
        #[command(subcommand)]
        command: SparkCommand,
    },
    /// Generate files from templates.
    File {
        #[command(subcommand)]
        command: FileCommand,
    },
    /// Inspect the current bkmt runtime configuration.
    Diagnostics {
        #[command(subcommand)]
        command: DiagnosticsCommand,
    },
    /// List tools compiled into this release.
    Tools {
        /// Emit stable machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Parser)]
#[command(
    name = "bkmt",
    version = toolbox_core::WORKSPACE_VERSION,
    about = "Bro Know My Toolbox command-line interface",
    arg_required_else_help = true,
    subcommand_required = true
)]
pub(crate) struct Cli {
    /// Override the shared GUI/CLI data directory.
    #[arg(long, global = true)]
    pub(crate) data_dir: Option<PathBuf>,
    /// Use a data directory beside the executable package.
    #[arg(long, global = true)]
    pub(crate) portable: bool,
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

fn parse_variable(value: &str) -> Result<(String, String), String> {
    let Some((name, value)) = value.split_once('=') else {
        return Err("variables must use NAME=VALUE syntax".to_string());
    };
    if name.is_empty() {
        return Err("variable name cannot be empty".to_string());
    }
    Ok((name.to_string(), value.to_string()))
}
