mod file;
mod spark;

use crate::args::{Cli, Command, ConsentCommand, DiagnosticsCommand};
use crate::error::CliError;
use crate::runtime::{parse_capability, resolve_data_root};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolOutput<'a> {
    id: &'a str,
    cli_namespace: &'a str,
}

#[derive(serde::Serialize)]
struct DataDirOutput {
    path: String,
    source: toolbox_core::DataRootSource,
}

pub(crate) async fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Some(Command::Spark { command }) => {
            spark::run(command, cli.data_dir, cli.portable).await?;
        }
        Some(Command::File { command }) => {
            file::run(command, cli.data_dir, cli.portable)?;
        }
        Some(Command::Consent { command }) => {
            run_consent(command, cli.data_dir, cli.portable)?;
        }
        Some(Command::Diagnostics {
            command: DiagnosticsCommand::DataDir { json },
        }) => {
            let resolved = resolve_data_root(cli.data_dir, cli.portable, json)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&DataDirOutput {
                        path: resolved
                            .path
                            .to_str()
                            .ok_or_else(|| {
                                CliError::operation(
                                    "data_root_not_unicode",
                                    "the data directory cannot be represented as UTF-8",
                                    1,
                                    json,
                                )
                            })?
                            .to_owned(),
                        source: resolved.source,
                    })
                    .map_err(|error| {
                        CliError::operation(
                            "serialization_failed",
                            format!("failed to serialize data directory: {error}"),
                            1,
                            json,
                        )
                    })?
                );
            } else {
                println!("{}", resolved.path.display());
            }
        }
        Some(Command::Tools { json }) => run_tools(json)?,
        None => {}
    }
    Ok(())
}

fn run_consent(
    command: ConsentCommand,
    data_dir: Option<std::path::PathBuf>,
    portable: bool,
) -> Result<(), CliError> {
    let (tool_id, capability, decision, label) = match command {
        ConsentCommand::Allow {
            tool_id,
            capability,
        } => (
            tool_id,
            capability,
            toolbox_core::ConsentDecision::Allowed,
            "allowed",
        ),
        ConsentCommand::Deny {
            tool_id,
            capability,
        } => (
            tool_id,
            capability,
            toolbox_core::ConsentDecision::Denied,
            "denied",
        ),
    };
    let data_root = resolve_data_root(data_dir, portable, false)?;
    let capability = parse_capability(&capability)
        .map_err(|error| CliError::input("invalid_capability", error, false))?;
    let mut authorizer = toolbox_core::CapabilityAuthorizer::new(
        toolbox_core::FileConsentStore::new(&data_root.path),
    )
    .map_err(|error| CliError::operation("authorization_failed", error.to_string(), 1, false))?;
    authorizer
        .record(&tool_id, capability, decision)
        .map_err(|error| match error {
            toolbox_core::AuthorizationError::UnknownTool(_)
            | toolbox_core::AuthorizationError::CapabilityNotDeclared { .. } => {
                CliError::input("invalid_consent_target", error.to_string(), false)
            }
            _ => CliError::operation("authorization_failed", error.to_string(), 1, false),
        })?;
    println!("{label}");
    Ok(())
}

fn run_tools(json: bool) -> Result<(), CliError> {
    let tools = toolbox_core::embedded_tool_catalog().map_err(|error| {
        CliError::operation(
            "tool_catalog_unavailable",
            format!("failed to load the built-in tool catalog: {error}"),
            1,
            json,
        )
    })?;
    if json {
        let output = tools
            .iter()
            .map(|tool| ToolOutput {
                id: &tool.id,
                cli_namespace: &tool.cli_namespace,
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            serde_json::to_string(&output).map_err(|error| {
                CliError::operation(
                    "serialization_failed",
                    format!("failed to serialize tool catalog: {error}"),
                    1,
                    json,
                )
            })?
        );
    } else {
        for tool in tools {
            println!("{}\t{}", tool.cli_namespace, tool.id);
        }
    }
    Ok(())
}
