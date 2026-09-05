use std::collections::BTreeSet;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
pub enum CapabilityId {
    #[serde(rename = "credentials:ai")]
    CredentialsAi,
    #[serde(rename = "filesystem:read")]
    FilesystemRead,
    #[serde(rename = "filesystem:write")]
    FilesystemWrite,
    #[serde(rename = "network:spark")]
    NetworkSpark,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDescriptor {
    pub id: String,
    pub title_key: String,
    pub description_key: String,
    pub route: String,
    pub route_name: String,
    pub cli_namespace: String,
    pub keywords: Vec<String>,
    pub capabilities: Vec<CapabilityId>,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolCatalogError {
    #[error("invalid tool catalog JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("invalid tool catalog: {0}")]
    InvalidDefinition(String),
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

fn parse_tool_catalog(source: &str) -> Result<Vec<ToolDescriptor>, ToolCatalogError> {
    let tools: Vec<ToolDescriptor> = serde_json::from_str(source)?;
    let mut ids = BTreeSet::new();
    let mut routes = BTreeSet::new();
    let mut route_names = BTreeSet::new();
    let mut namespaces = BTreeSet::new();
    for tool in &tools {
        let route_slug = tool.route.strip_prefix("/tools/").unwrap_or_default();
        let valid_route_name = tool
            .route_name
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase())
            && tool
                .route_name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric());
        let unique_capabilities = tool.capabilities.iter().collect::<BTreeSet<_>>();
        if !valid_slug(&tool.id)
            || !valid_slug(route_slug)
            || !valid_route_name
            || !valid_slug(&tool.cli_namespace)
            || tool.title_key.trim().is_empty()
            || tool.description_key.trim().is_empty()
            || tool
                .keywords
                .iter()
                .any(|keyword| keyword.trim().is_empty())
            || unique_capabilities.len() != tool.capabilities.len()
        {
            return Err(ToolCatalogError::InvalidDefinition(tool.id.clone()));
        }
        if !ids.insert(&tool.id)
            || !routes.insert(&tool.route)
            || !route_names.insert(&tool.route_name)
            || !namespaces.insert(&tool.cli_namespace)
        {
            return Err(ToolCatalogError::InvalidDefinition(format!(
                "duplicate declaration for {}",
                tool.id
            )));
        }
    }
    Ok(tools)
}

pub fn embedded_tool_catalog() -> Result<Vec<ToolDescriptor>, ToolCatalogError> {
    parse_tool_catalog(include_str!("../../../packages/tool-contract/catalog.json"))
}

#[cfg(test)]
mod tests {
    use super::parse_tool_catalog;

    #[test]
    fn rust_catalog_boundary_rejects_invalid_routes_and_duplicate_capabilities() {
        let invalid = r#"[{
          "id":"file-generator",
          "titleKey":"tool.file_generator.name",
          "descriptionKey":"tool.file_generator.description",
          "route":"/settings",
          "routeName":"ToolFileGenerator",
          "cliNamespace":"file",
          "keywords":["file"],
          "capabilities":["filesystem:write","filesystem:write"]
        }]"#;

        assert!(parse_tool_catalog(invalid).is_err());
    }
}
