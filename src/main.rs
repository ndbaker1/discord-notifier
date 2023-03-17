use std::collections::HashMap;

use anyhow::Result;
use structopt::StructOpt;

#[derive(structopt::StructOpt)]
struct Args {
    /// Discord User ID
    #[structopt(short = "u", long, env = "DISCORD_USER_ID")]
    user: String,
    /// Discord Bot Token
    #[structopt(short = "t", long, env = "DISCORD_BOT_TOKEN")]
    token: String,
    /// Additional header info for the message
    #[structopt(short = "i", long)]
    info: Option<String>,
}

fn main() -> Result<()> {
    let Args { user, token, info } = Args::from_args();

    // read the entire stdin buffer to use as the output message
    let input = if atty::is(atty::Stream::Stdin) {
        std::io::stdin()
            .lines()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "".to_string()
    };

    let content = info
        .map(|info| format!("> {info}\n```\n{input}\n```"))
        .unwrap_or(input);

    send_message(&token, &user, &content)
}

fn send_message(token: &str, user: &str, content: &str) -> Result<()> {
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
