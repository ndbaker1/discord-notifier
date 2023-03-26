use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

const APP_NAME: &'static str = "discord-notifier";

macro_rules! read_stdin_string {
    () => {
        std::io::stdin()
            .lines()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n")
    };
}

/// Send a message to a discord channel using the Discord Bot api (mixed versions)
#[derive(structopt::StructOpt)]
struct CliArgs {
    /// channel or user id (depending on if the message is going to a DM or not)
    #[structopt(short = "c", long, env = "DISCORD_CHANNEL_ID")]
    channel_id: Option<String>,
    /// token of your discord bot
    #[structopt(short = "t", long, env = "DISCORD_BOT_TOKEN")]
    token: Option<String>,
    /// prepend additional text to the message
    #[structopt(short = "p", long)]
    prepend_message: Option<String>,
    /// read stdin to construct the message content
    #[structopt(short = "i", long)]
    stdin: bool,
    /// send the message to a direct message channel. (will not work if mismatched)
    #[structopt(short = "d", long)]
    dm: bool,
    /// initialize a config file containing default values for optional fields
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
                channel: Some(
                    prompt_enter("enter default channel ID: ")?
                        .trim()
                        .to_string(),
                ),
                token: Some(
                    prompt_enter("enter default Bot Token: ")?
                        .trim()
                        .to_string(),
                ),
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
            true => "\n\n".to_owned() + &read_stdin_string!(),
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

    match cli_args.dm {
        true => send_message_dm(token, channel, content, &client),
        false => send_message_channel(token, channel, content, &client),
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

    // hoist up any request errors by trying to unwrap the content
    channel_setup_response.error_for_status_ref()?;

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
        .send()?
        .error_for_status_ref()?;

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
        .send()?
        .error_for_status_ref()?;

    Ok(())
}
