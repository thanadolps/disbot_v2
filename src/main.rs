#![deny(unused_must_use)]
mod braille;
mod pyremote;

mod fibo;
mod unicode;

use image::DynamicImage;
use poise::command;
use poise::samples::register_in_guild;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Attachment;
use poise::serenity_prelude::AttachmentType;
use poise::serenity_prelude::CreateAttachment;
use poise::serenity_prelude::FutureExt;
use poise::serenity_prelude::GuildId;
use poise::CodeBlock;
use poise::CreateReply;
use rand::thread_rng;
use rug::ops;
use serenity::GatewayIntents;
// use serenity::client::{Client, Context, EventHandler};
// use serenity::framework::standard::{
//     help_commands,
//     macros::{group, help},
//     Args, CommandError, CommandGroup, CommandResult, HelpOptions, StandardFramework,
// };
use serenity::model::channel::Message;
use std::borrow::Cow;
use std::ops::Deref;
use std::time::Duration;
use tokio::time::MissedTickBehavior;

use color_eyre::Result;
// use ibig::UBig;
use image::GenericImageView;
use itertools::Itertools;
use rand::prelude::*;
// use serenity::builder::CreateMessage;
use serenity::futures::{
    future::{self, Either},
    pin_mut, StreamExt,
};
// use serenity::model::prelude::{AttachmentType, Channel, ChannelType, User, UserId};
// use serenity::prelude::GatewayIntents;
// use serenity::static_assertions::_core::time::Duration;
// use serenity::utils::MessageBuilder;
use std::collections::HashSet;
use std::env;
use std::fmt::Display;
use std::future::Future;
use std::process::Output;
use std::str::from_utf8;

const DISCORD_MESSAGE_LIMIT: usize = 2000;
const DISCORD_WIDTH_LIMIT: usize = 60;

struct Data {}
type Error = color_eyre::eyre::Error;
type Context<'a> = poise::Context<'a, Data, Error>;

// #[group]
// #[commands(hello, count, unicode, fibo, py, swap_channel)]
// struct General;

// struct Handler;

// #[async_trait]
// impl EventHandler for Handler {}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN envar should be set");
    // let intents = GatewayIntents::GUILD_MESSAGES
    //     | GatewayIntents::DIRECT_MESSAGES
    //     | GatewayIntents::MESSAGE_reply;
    let intents = GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                hello(),
                count(),
                fibo::fibo(),
                py(),
                repeat(),
                unicode::unicode(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    println!("Starting bot");
    client?.start().await?;
    Ok(())
}

// #[help]
// async fn help_cmd(
//     context: &Context,
//     msg: &Message,
//     args: Args,
//     help_options: &'static HelpOptions,
//     groups: &[&'static CommandGroup],
//     owners: HashSet<UserId>,
// ) -> CommandResult {
//     let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
//     Ok(())
// }

/// Force bot to greet you
#[command(prefix_command, slash_command)]
async fn hello(ctx: Context<'_>) -> Result<()> {
    let greeting = {
        [
            "hello",
            "hi",
            "what's up",
            "good day",
            "how are you",
            "howdy",
            "greetings",
            "bonjour",
            "hola",
            "こんにちは",
        ]
        .choose(&mut rand::rng())
        .expect("not empty slice")
    };

    let reply = format!("{} {}", greeting, ctx.author());
    ctx.reply(reply).await?;
    Ok(())
}

/// Make the bot count for you (a timer)
#[command(prefix_command, slash_command)]
async fn count(ctx: Context<'_>, second: u8, weeb: Option<bool>) -> Result<()> {
    let message = ctx.reply("Counting...").await?;
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let weeb = weeb.unwrap_or(false);
    let reply = if weeb {
        |sec| format!("{}秒経過！", sec)
    } else {
        |sec| format!("{} second has passed", sec)
    };

    for i in 1..=second {
        interval.tick().await;
        let reply = format!("{}...", reply(i));
        message
            .edit(ctx, CreateReply::default().content(reply))
            .await?;
    }

    let last_reply = if weeb {
        format!("**{}！。** やった、終わったのだ", reply(second))
    } else {
        format!("**{}**", reply(second))
    };
    message
        .edit(ctx, CreateReply::default().content(last_reply))
        .await?;

    Ok(())
}

/// Run python code
///
/// usage: |py ```python_code```|
#[command(prefix_command)]
async fn py(ctx: Context<'_>, #[rest] code: String) -> Result<()> {
    async fn send_as_attachment(ctx: Context<'_>, stdout: Vec<u8>, stderr: Vec<u8>) -> Result<()> {
        let mut reply = CreateReply::default().reply(true);
        if !stdout.is_empty() {
            reply = reply.attachment(CreateAttachment::bytes(stdout, "stdout.txt"));
        }
        if !stderr.is_empty() {
            reply = reply.attachment(CreateAttachment::bytes(stderr, "stderr.txt"));
        }

        ctx.send(reply).await?;
        Ok(())
    }

    let mut code = code.trim();
    if code.starts_with("```") && code.ends_with("```") {
        code = &code[3..code.len() - 3];
    } else if code.starts_with('`') && code.ends_with('`') {
        code = &code[1..code.len() - 1];
    }
    ctx.defer_or_broadcast().await?;

    // run python code
    let output = match pyremote::secure_run_python_code(&code, Duration::from_secs(5)).await {
        Ok(output) => output,
        Err(pyremote::Error::Timeout { timeout }) => {
            ctx.reply(format!("Code Timeout in {} seconds", timeout.as_secs()))
                .await?;
            return Ok(());
        }
        Err(pyremote::Error::IO(e)) => {
            return Err(e.into());
        }
    };

    // create reply string
    let stdout = output.stdout.trim_ascii();
    let stderr = output.stderr.trim_ascii();

    let (Ok(stdout), Ok(stderr)) = (str::from_utf8(stdout), str::from_utf8(stderr)) else {
        // message content non-utf8 bytes, send as attachment
        send_as_attachment(ctx, stdout.into(), stderr.into()).await?;
        return Ok(());
    };

    let report = itertools::join(
        [stdout, stderr]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| format!("```{}```", s)), // warp output in code block
        "\n",
    );

    match report.chars().count() {
        0 => {
            ctx.reply("*<empty output>*").await?;
        }
        1..=DISCORD_MESSAGE_LIMIT => {
            ctx.reply(report).await?;
        }
        _ => {
            // message too large, send as attachment
            send_as_attachment(ctx, stdout.into(), stderr.into()).await?;
        }
    }
    Ok(())
}

#[command(prefix_command, slash_command)]
async fn repeat(ctx: Context<'_>, c: char, n: u32) -> Result<()> {
    let buf = c.to_string().repeat(n as usize);
    ctx.reply(buf).await?;
    Ok(())
}

// #[command]
// async fn swap_channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     let author = &msg.author;

//     // Get all channels
//     let mut channels = {
//         let channel = msg.channel(ctx).await?;
//         let parent_channel = channel
//             .guild()
//             .unwrap()
//             .parent_id
//             .unwrap()
//             .to_channel(ctx)
//             .await?;
//         let category = parent_channel.category().unwrap(); // as in folder.
//         category.guild_id.channels(ctx).await?
//     };

//     // Filter only voice channels
//     let mut vcs = channels
//         .values_mut()
//         .filter(|channel| channel.kind == ChannelType::Voice)
//         .collect_vec();

//     // Find voice channel that user is in
//     // let mut vc1 = None;

//     for (_, vc) in channels {
//         dbg!(vc.members(ctx).await.ok());
//     }

//     /*for vc in vcs {
//         if vc
//             .members(ctx)
//             .await
//             .expect("voice channel to have member list")
//             .iter()
//             .any(|member| &member.user == author)
//         {
//             vc1 = Some(vc);
//         }
//     }

//     let vc1 = match vc1 {
//         Some(vc) => vc,
//         None => {
//             msg.reply(ctx, "Cannot find voice channel that user is in")
//                 .await?;
//             return Ok(());
//         }
//     };

//     dbg!(vc1);*/
//     /*
//     for vc in vcs {
//         let name = vc.name();
//         let new_name = format!("{}++", name);
//         dbg!(&name);
//         dbg!(&new_name);
//         vc.edit(ctx, |e| e.name(new_name)).await?;
//     }*/
//     // dbg!(guild.channels);
//     Ok(())
// }
