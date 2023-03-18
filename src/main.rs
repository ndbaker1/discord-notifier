use std::collections::HashMap;

use anyhow::Result;
use structopt::StructOpt;

#[derive(structopt::StructOpt)]
struct Args {
    /// Discord User ID or Channel to send a message to
    #[structopt(short = "u", long, env = "DISCORD_USER_ID")]
    user: String,
    /// Discord Bot Token
    #[structopt(short = "t", long, env = "DISCORD_BOT_TOKEN")]
    token: String,
    /// Additional header info for the message
    #[structopt(short = "i", long)]
    info: Option<String>,
    /// Flag to control whether stdin should be read to contruct the message
    #[structopt(short = "r", long)]
    read_stdin: bool,
    /// Whether or not the message is a Direct Message or not
    #[structopt(short = "d", long)]
    dm: bool,
}

fn main() -> Result<()> {
    let Args {
        user,
        token,
        info,
        read_stdin,
        dm,
    } = Args::from_args();

    // read the entire stdin buffer to use as the output message
    let input = if read_stdin {
        std::io::stdin()
            .lines()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "job completed.".to_string()
    };

    let content = info
        .map(|info| format!("> {info}\n```\n{input}\n```"))
        .unwrap_or(input);

    if dm {
        send_message_dm(&token, &user, &content)
    } else {
        send_message_channel(&token, &user, &content)
    }
}

fn send_message_dm(token: &str, user: &str, content: &str) -> Result<()> {
    // setup a client to use for HTTP requests
    let client = reqwest::blocking::Client::new();

    let channel_setup_response = client
        .post("https://discord.com/api/v9/users/@me/channels")
        .json(&HashMap::<_, _>::from_iter(
            [("recipient_id", user)].into_iter(),
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

fn send_message_channel(token: &str, channel_id: &str, content: &str) -> Result<()> {
    // setup a client to use for HTTP requests
    let client = reqwest::blocking::Client::new();

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
