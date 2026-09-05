use std::io::Read;
use std::path::PathBuf;

use crate::args::SparkCommand;
use crate::error::CliError;
use crate::runtime::{check_tool_capability, resolve_data_root};

const MAX_SPARK_REPORT_BYTES: u64 = 64 * 1024 * 1024;

fn is_regular_report(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return false;
        }
    }
    metadata.is_file()
}

fn open_report(source: &std::path::Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    options.open(source)
}

fn load_report(
    source: &std::path::Path,
    text: bool,
    json: bool,
    data_root: &toolbox_core::ResolvedDataRoot,
) -> Result<bkmsa_core::Report, CliError> {
    check_tool_capability(
        data_root,
        "spark-analyzer",
        toolbox_core::CapabilityId::FilesystemRead,
        "filesystem:read",
        json,
    )?;
    let mut file = open_report(source).map_err(|error| {
        CliError::input(
            "report_read_failed",
            format!("failed to read Spark report: {error}"),
            json,
        )
    })?;
    let metadata = file.metadata().map_err(|error| {
        CliError::input(
            "report_read_failed",
            format!("failed to inspect Spark report: {error}"),
            json,
        )
    })?;
    if !is_regular_report(&metadata) {
        return Err(CliError::input(
            "invalid_report_source",
            "Spark report source must be a regular file",
            json,
        ));
    }
    if metadata.len() > MAX_SPARK_REPORT_BYTES {
        return Err(CliError::input(
            "report_too_large",
            "Spark report exceeds the 64 MiB input limit",
            json,
        ));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.by_ref()
        .take(MAX_SPARK_REPORT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            CliError::input(
                "report_read_failed",
                format!("failed to read Spark report: {error}"),
                json,
            )
        })?;
    if bytes.len() as u64 > MAX_SPARK_REPORT_BYTES {
        return Err(CliError::input(
            "report_too_large",
            "Spark report exceeds the 64 MiB input limit",
            json,
        ));
    }
    let source_name = source.to_string_lossy().into_owned();
    if text {
        let content = String::from_utf8(bytes)
            .map_err(|error| CliError::input("invalid_text_report", error.to_string(), json))?;
        bkmsa_core::parse_text_report(content, source_name)
    } else {
        let hint = source
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        bkmsa_core::parse_report_bytes(&bytes, source_name, hint)
    }
    .map_err(|error| CliError::input("report_decode_failed", error.to_string(), json))
}

fn tool_arguments(
    complete: Option<&str>,
    pairs: &[String],
    json_output: bool,
) -> Result<serde_json::Value, CliError> {
    if let Some(complete) = complete {
        let value: serde_json::Value = serde_json::from_str(complete).map_err(|error| {
            CliError::input("invalid_tool_args", error.to_string(), json_output)
        })?;
        if !value.is_object() {
            return Err(CliError::input(
                "invalid_tool_args",
                "--args must be a JSON object",
                json_output,
            ));
        }
        return Ok(value);
    }
    let mut arguments = serde_json::Map::new();
    for pair in pairs {
        let Some((key, raw)) = pair.split_once('=') else {
            return Err(CliError::input(
                "invalid_tool_args",
                format!("invalid --arg '{pair}'; expected KEY=VALUE"),
                json_output,
            ));
        };
        if key.trim().is_empty() || arguments.contains_key(key.trim()) {
            return Err(CliError::input(
                "invalid_tool_args",
                format!("invalid or duplicate tool argument: {key}"),
                json_output,
            ));
        }
        arguments.insert(
            key.trim().to_string(),
            serde_json::from_str(raw)
                .unwrap_or_else(|_| serde_json::Value::String(raw.to_string())),
        );
    }
    Ok(serde_json::Value::Object(arguments))
}

pub(crate) async fn run(
    command: SparkCommand,
    data_dir: Option<PathBuf>,
    portable: bool,
) -> Result<(), CliError> {
    match command {
        SparkCommand::Tools { json } => {
            let tools = bkmsa_core::report_tool_descriptions();
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&tools).map_err(|error| CliError::operation(
                        "serialization_failed",
                        format!("failed to serialize Spark tools: {error}"),
                        1,
                        json
                    ))?
                );
            } else {
                for tool in tools {
                    println!("{}\t{}", tool.name, tool.description);
                }
            }
        }
        SparkCommand::Inspect { source, text, json } => {
            let data_root = resolve_data_root(data_dir, portable, json)?;
            let report = load_report(&source, text, json, &data_root)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "command": "inspect",
                        "source": report.source,
                        "kind": report.kind.as_str(),
                        "summary": report.summary,
                    })
                );
            } else {
                println!("{}\t{}", report.kind.as_str(), report.summary.title);
            }
        }
        SparkCommand::Tool {
            source,
            tool,
            text,
            args,
            arguments,
            json,
        } => {
            let data_root = resolve_data_root(data_dir, portable, json)?;
            let report = load_report(&source, text, json, &data_root)?;
            let tool = tool.replace('-', "_");
            let arguments = tool_arguments(args.as_deref(), &arguments, json)?;
            let result = bkmsa_core::execute_tool(&report, &tool, arguments)
                .map_err(|error| CliError::input("spark_tool_failed", error.to_string(), json))?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "command": tool,
                        "source": report.source,
                        "kind": report.kind.as_str(),
                        "result": result,
                    })
                );
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).map_err(|error| CliError::operation(
                        "serialization_failed",
                        format!("failed to render Spark tool result: {error}"),
                        1,
                        json
                    ))?
                );
            }
        }
        SparkCommand::Analyze {
            source,
            text,
            base_url,
            model,
            temperature,
            max_rounds,
            json,
        } => {
            let data_root = resolve_data_root(data_dir, portable, json)?;
            let report = load_report(&source, text, json, &data_root)?;
            check_tool_capability(
                &data_root,
                "spark-analyzer",
                toolbox_core::CapabilityId::NetworkSpark,
                "network:spark",
                json,
            )?;
            check_tool_capability(
                &data_root,
                "spark-analyzer",
                toolbox_core::CapabilityId::CredentialsAi,
                "credentials:ai",
                json,
            )?;
            let api_key = std::env::var("BKMT_SPARK_API_KEY")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .or_else(|| {
                    std::env::var("BKMSA_API_KEY")
                        .ok()
                        .filter(|value| !value.trim().is_empty())
                })
                .ok_or_else(|| {
                    CliError::input(
                        "missing_ai_credential",
                        "BKMT_SPARK_API_KEY or BKMSA_API_KEY is required for Spark AI analysis",
                        json,
                    )
                })?;
            let config = bkmsa_agent::AiConfig::new(base_url, api_key, model, temperature)
                .map_err(|error| CliError::input("invalid_ai_config", error.to_string(), json))?;
            let client = bkmsa_agent::OpenAiClient::new(config)
                .map_err(|error| CliError::input("invalid_ai_config", error.to_string(), json))?;
            let result = bkmsa_agent::run_analysis(
                &report,
                &client,
                bkmsa_agent::AgentOptions {
                    max_rounds,
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| {
                CliError::operation("spark_analysis_failed", error.to_string(), 5, json)
            })?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&result).map_err(|error| CliError::operation(
                        "serialization_failed",
                        format!("failed to serialize Spark analysis: {error}"),
                        1,
                        json
                    ))?
                );
            } else {
                println!("{}", result.diagnosis);
            }
        }
    }
    Ok(())
}
