use crate::auth;
use crate::mcp;
use anyhow::Result;
use clap::ArgMatches;

/// Handle the `init` subcommand: prompt for bot credentials, persist them, and verify via MCP config fetch.
pub async fn handle_init_cmd(_matches: &ArgMatches) -> Result<()> {
    cliclack::intro("企业微信机器人初始化")?;

    // 交互选择接入方式
    let method: &str = cliclack::select("请选择企微机器人接入方式：")
        .item("qrcode", "扫码接入（推荐）", "")
        .item("manual", "手动输入 Bot ID 和 Secret", "")
        .interact()?;

    let bot = match method {
        "qrcode" => init_qrcode().await?,
        _ => init_manual().await?,
    };

    auth::set_bot_info(&bot)?;
    verify_and_finish().await
}

/// 扫码接入流程
async fn init_qrcode() -> Result<auth::Bot> {
    auth::scan_qrcode_for_bot().await
}

/// 手动输入 Bot ID 和 Secret
async fn init_manual() -> Result<auth::Bot> {
    let bot_id: String = cliclack::input("企业微信机器人 Bot ID")
        .placeholder("请输入企业微信机器人ID")
        .interact()?;

    let bot_secret: String = cliclack::password("企业微信机器人 Secret")
        .mask('*')
        .interact()?;

    Ok(auth::Bot::new(bot_id, bot_secret))
}

/// 验证凭证并完成初始化
async fn verify_and_finish() -> Result<()> {
    let spinner = cliclack::spinner();
    spinner.start("正在验证企业微信机器人凭证...");

    if let Err(e) = mcp::config::fetch_mcp_config().await {
        spinner.stop("企业微信机器人凭证验证失败");

        let mut output_errmsg: String = "验证企业微信机器人凭证失败".to_owned();

        match &e {
            mcp::error::FetchMcpConfigError::Api(resp) => {
                if let Some(ref msg) = resp.errmsg {
                    if !msg.is_empty() {
                        output_errmsg = msg.clone();
                    }
                }
            }
            mcp::error::FetchMcpConfigError::Http(http_err) => {
                output_errmsg = format!("{} HTTP返回状态码 {}", output_errmsg, http_err.status);
            }
            mcp::error::FetchMcpConfigError::Other(other_err) => {
                output_errmsg = other_err.to_string();
            }
        }

        // Credentials invalid or server unreachable — rollback
        auth::clear_bot_info();
        mcp::config::clear_mcp_config();
        cliclack::outro("初始化失败 ❌")?;
        anyhow::bail!(output_errmsg);
    }

    spinner.stop("企业微信机器人凭证验证成功");
    cliclack::outro("初始化完成 ✅")?;
    Ok(())
}
