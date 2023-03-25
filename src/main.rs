use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

const APP_NAME: &'static str = "discord-notifier";

#[derive(structopt::StructOpt)]
struct CliArgs {
    /// Discord User ID or Channel to send a message to
    #[structopt(short = "c", long, env = "DISCORD_CHANNEL_ID")]
    channel_id: Option<String>,
    /// Discord Bot Token
    #[structopt(short = "t", long, env = "DISCORD_BOT_TOKEN")]
    token: Option<String>,
    /// Additional header info for the message
    #[structopt(short = "p", long)]
    prepend_message: Option<String>,
    /// Flag to control whether stdin should be read to contruct the message
    #[structopt(short = "i", long)]
    stdin: bool,
    /// Whether or not the message is a Direct Message or not
    #[structopt(short = "d", long)]
    dm: bool,

    /// Whether or to initialize a config file
    #[structopt(long)]
    init: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct ConfigArgs {
    /// Discord User ID or Channel to send a message to
    channel: Option<String>,
    /// Discord Bot Token
    token: Option<String>,
}

fn main() -> Result<()> {
    let cli_args = CliArgs::from_args();

    if cli_args.init {
        confy::store(
            APP_NAME,
            None,
            ConfigArgs {
                channel: Some(prompt_enter("enter default channel ID: ")?),
                token: Some(prompt_enter("enter default Bot Token: ")?),
            },
        )?;

        return Ok(());
    }

    let config_args = confy::get_configuration_file_path(APP_NAME, None)?
        .exists()
        .then_some(confy::load::<ConfigArgs>(APP_NAME, None)?)
        .or(None);

    // read the entire stdin buffer to use as the output message
    let input = format!(
        "completed at {}{}",
        Utc::now().to_rfc2822(),
        match cli_args.stdin {
            true => "\n\n".to_owned() + &read_stdin_string(),
            false => "".to_owned(),
        }
    );

    let content = &cli_args
        .prepend_message
        .map(|info| format_message(&info, &input))
        .unwrap_or(input);

    let token = &cli_args
        .token
        .or(config_args
            .as_ref()
            .map(|c| c.token.clone())
            .unwrap_or_default())
        .ok_or_else(|| anyhow!("no token provided."))?;

    let channel = &cli_args
        .channel_id
        .or(config_args
            .as_ref()
            .map(|c| c.channel.clone())
            .unwrap_or_default())
        .ok_or_else(|| anyhow!("no channel provided."))?;

    // setup a client to use for HTTP requests
    let client = reqwest::blocking::Client::new();

    if cli_args.dm {
        send_message_dm(token, channel, content, &client)
    } else {
        send_message_channel(token, channel, content, &client)
    }
}

fn prompt_enter(prompt: &str) -> Result<String> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut line = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_line(&mut line)?;
    Ok(line)
}

fn format_message(header: &str, input: &str) -> String {
    format!("> {header}\n```\n{input}\n```")
}

fn read_stdin_string() -> String {
    std::io::stdin()
        .lines()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n")
}

fn send_message_dm(
    token: &str,
    user_id: &str,
    content: &str,
    client: &reqwest::blocking::Client,
) -> Result<()> {
    let channel_setup_response = client
        .post("https://discord.com/api/v9/users/@me/channels")
        .json(&HashMap::<_, _>::from_iter(
            [("recipient_id", user_id)].into_iter(),
        ))
        .header("authorization", format!("Bot {token}"))
        .send()?;

    let channel_id = channel_setup_response
        .json::<serde_json::Value>()?
        .get("id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    client
        .post(format!(
            "https://discord.com/api/v8/channels/{channel_id}/messages",
        ))
        .json(&HashMap::<_, _>::from_iter(
            [("content", content)].into_iter(),
        ))
        .header("authorization", format!("Bot {token}"))
        .send()?;

    Ok(())
}

fn send_message_channel(
    token: &str,
    channel_id: &str,
    content: &str,
    client: &reqwest::blocking::Client,
) -> Result<()> {
    client
        .post(format!(
            "https://discord.com/api/v8/channels/{channel_id}/messages",
        ))
        .json(&HashMap::<_, _>::from_iter(
            [("content", content)].into_iter(),
        ))
        .header("authorization", format!("Bot {token}"))
        .send()?;

    Ok(())
}
