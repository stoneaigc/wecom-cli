use crate::{config, json_rpc};
use anyhow::Result;

/// Fetch and display the available tools for a given category via JSON-RPC.
pub async fn show_category_tools(category_name: &str) -> Result<()> {
    // Try to dynamically fetch the tool list via JSON-RPC
    let res = json_rpc::send(category_name, "tools/list", None, None).await?;

    // Parse the returned tool list
    let Some(tools) = res
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|r| r.as_array())
    else {
        anyhow::bail!("无法获取 {} 品类的工具列表: {res}", category_name);
    };

    // Get category description
    let description = config::get_categories()
        .iter()
        .find(|info| info.name == category_name)
        .map(|info| info.description)
        .unwrap_or("");

    let header = clap::builder::styling::Style::new().bold().underline();
    let bold = clap::builder::styling::Style::new().bold();

    println!(
        "{header}Usage:{header:#} {bold}{} {category_name}{bold:#} [COMMAND] [ARGS]",
        env!("CARGO_BIN_NAME")
    );
    println!();
    println!("{description}");

    if !tools.is_empty() {
        println!();
        println!("{header}## Commands{header:#}");
        for tool in tools {
            let Some(name) = tool.get("name").and_then(|n| n.as_str()) else {
                continue;
            };
            println!();
            println!("{header}### {}{header:#}", name);
            if let Some(description) = tool.get("description").and_then(|d| d.as_str()) {
                println!();
                println!("{description}");
            }
        }
    }

    Ok(())
}

pub async fn show_tool_help(category_name: &str, tool_name: &str) -> Result<()> {
    // If not in cache, fetch dynamically
    let res = json_rpc::send(category_name, "tools/list", None, None).await?;

    // Parse the returned tool list
    let Some(tools) = res
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|r| r.as_array())
    else {
        anyhow::bail!("无法获取 {} 品类的工具列表: {res}", category_name);
    };

    // Find specific tool
    let Some(tool) = tools.iter().find(|t| {
        t.get("name")
            .and_then(|n| n.as_str())
            .map(|name| name == tool_name)
            .unwrap_or(false)
    }) else {
        anyhow::bail!("未找到 {category_name}.{tool_name} 工具");
    };

    let header = clap::builder::styling::Style::new().bold().underline();
    let bold = clap::builder::styling::Style::new().bold();

    println!(
        "{header}Usage:{header:#} {bold}{} {category_name} {tool_name}{bold:#} [ARGS]",
        env!("CARGO_BIN_NAME")
    );

    if let Some(description) = tool.get("description").and_then(|d| d.as_str()) {
        println!();
        println!("{}", description);
    }

    if let Some(input_schema) = tool.get("inputSchema") {
        println!();
        println!("{header}Input Schema:{header:#}");
        println!();
        println!("```json");
        println!(
            "{}",
            serde_json::to_string_pretty(input_schema).unwrap_or_default()
        );
        println!("```");
    }

    Ok(())
}
