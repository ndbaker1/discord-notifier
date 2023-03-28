<p align="center">
	<h1 align="center">discord-notifier</h1>
</p>
<p align="center">Easily send the output of long task/program to a discord channel</p>

## Installation 🛠

Download the [latest release](https://github.com/ndbaker1/discord-notifier/releases/tag/latest) for your platform and Unzip the executable into any installation directory
( `PATH` directories or script directory of your choice )

## Usage ⚡

```bash
discord-notifier -c <CHANNEL_ID> -t <BOT_TOKEN>
```

> You will need to share a server with the indicated Discord bot.  
> See [Discord docs](https://discord.com/developers/docs/getting-started) for help with setting up a Discord bot.

use the `--help` command for details.

## Configuration ⚙

If you want to initialize a config file with defaults it will look like so:

```bash
$ discord-notifier --init
enter default channel ID: <CHANNEL_ID>
enter default Bot Token: <BOT_TOKEN>
```

and from then on you can call the program without any arguments:

```bash
$ discord-notifier
```

