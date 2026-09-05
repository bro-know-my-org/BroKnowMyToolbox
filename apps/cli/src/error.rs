#[derive(Debug)]
pub(crate) struct CliError {
    code: &'static str,
    message: String,
    details: Option<serde_json::Value>,
    pub(crate) exit_code: i32,
    json: bool,
}

impl CliError {
    pub(crate) fn input(code: &'static str, message: impl Into<String>, json: bool) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            exit_code: 2,
            json,
        }
    }

    pub(crate) fn consent_required(json: bool) -> Self {
        Self {
            code: "consent_required",
            message: "filesystem write consent is required".to_string(),
            details: Some(serde_json::json!({
                "toolId": "file-generator",
                "capability": "filesystem:write"
            })),
            exit_code: 4,
            json,
        }
    }

    pub(crate) fn consent_denied(json: bool) -> Self {
        Self {
            code: "consent_denied",
            message: "filesystem write consent was denied".to_string(),
            details: Some(serde_json::json!({
                "toolId": "file-generator",
                "capability": "filesystem:write"
            })),
            exit_code: 4,
            json,
        }
    }

    pub(crate) fn execution_status(
        status: file_generator::ExecutionStatus,
        json: bool,
    ) -> Option<Self> {
        let (code, message, exit_code) = match status {
            file_generator::ExecutionStatus::Complete => return None,
            file_generator::ExecutionStatus::Conflict => (
                "conflict",
                "one or more files conflict with existing files",
                3,
            ),
            file_generator::ExecutionStatus::PartialFailure => {
                ("partial_failure", "some files could not be generated", 5)
            }
            file_generator::ExecutionStatus::Failed => {
                ("execution_failed", "files could not be generated", 5)
            }
        };
        Some(Self {
            code,
            message: message.to_string(),
            details: None,
            exit_code,
            json,
        })
    }

    pub(crate) fn capability(
        code: &'static str,
        message: impl Into<String>,
        tool_id: &str,
        capability: &'static str,
        json: bool,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details: Some(serde_json::json!({
                "toolId": tool_id,
                "capability": capability,
            })),
            exit_code: 4,
            json,
        }
    }

    pub(crate) fn operation(
        code: &'static str,
        message: impl Into<String>,
        exit_code: i32,
        json: bool,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            exit_code,
            json,
        }
    }

    pub(crate) fn render(&self) -> String {
        if self.json {
            serde_json::json!({
                "code": self.code,
                "message": self.message,
                "details": self.details,
            })
            .to_string()
        } else {
            self.message.clone()
        }
    }
}

impl From<String> for CliError {
    fn from(message: String) -> Self {
        Self {
            code: "internal_error",
            message,
            details: None,
            exit_code: 1,
            json: false,
        }
    }
}
