use std::env;

use anyhow::{bail, Context};
use futures::StreamExt;
use telegram_bot::CanReplySendMessage;
use telegram_bot::{Api, Message, MessageKind, MessageOrChannelPost, Update};

use sedboy::parse_sed_command;
use sedboy::SedCommand;

trait MessageExt {
    fn text(&self) -> Option<&str>;
}

impl MessageExt for Message {
    fn text(&self) -> Option<&str> {
        match &self.kind {
            MessageKind::Text { data, .. } => Some(data),
            _ => None,
        }
    }
}

trait MessageOrChannelPostExt {
    fn message(&self) -> Option<&Message>;
}

impl MessageOrChannelPostExt for MessageOrChannelPost {
    fn message(&self) -> Option<&Message> {
        match self {
            MessageOrChannelPost::Message(msg) => Some(msg),
            MessageOrChannelPost::ChannelPost(_) => None,
        }
    }
}

/// `message_or_channel_post` what is being replied to
///
/// `message_text` is the text of the message that is replying
async fn handle_reply(
    api: &Api,
    message_or_channel_post: &MessageOrChannelPost,
    sed_command: SedCommand<'_>,
) -> anyhow::Result<()> {
    // The message being replied to and its text
    let message = message_or_channel_post.message();
    let text = message.and_then(MessageExt::text);

    let (message, text) = match (message, text) {
        (Some(message), Some(text)) => (message, text),
        // Message being replied to was a channel post or was a message but not a text message
        _ => return Ok(()),
    };

    let replaced = sed_command.execute(text)?;

    let reply = message.text_reply(replaced);

    api.send(reply).await?;

    Ok(())
}

async fn handle_message<'a>(api: &Api, msg: &'a Message) -> anyhow::Result<()> {
    let sed_command = match msg.text().map(parse_sed_command) {
        Some(Ok((_, sed_command))) => sed_command,
        // Message is not a text message or is a invalid sed command
        _ => return Ok(()),
    };

    match &msg.reply_to_message {
        // If the current message is replying to something, we'll try to apply our SedCommand in the message being replied to
        Some(message_or_channel_post) => {
            handle_reply(api, message_or_channel_post, sed_command).await
        }
        // If the current message isn't replying to anything, we'll try to apply our SedCommand to the previously sent message
        None => todo!(),
    }
}

#[inline(always)]
async fn handle_update(api: &Api, update: Update) -> anyhow::Result<()> {
    match update.kind {
        telegram_bot::UpdateKind::Message(msg) => handle_message(api, &msg)
            .await
            .with_context(|| "Failed to handle a message!"),
        telegram_bot::UpdateKind::Error(err) => bail!(err),
        telegram_bot::UpdateKind::Unknown => bail!("Received unknown update"),
        _ => Ok(()),
    }
}

async fn try_main() -> anyhow::Result<()> {
    let token = env::var("TELEGRAM_BOT_TOKEN").with_context(|| "TELEGRAM_BOT_TOKEN was not set")?;

    let api = Api::new(token);
    let mut update_stream = api.stream();

    while let Some(update) = update_stream.next().await {
        let update = update?;
        handle_update(&api, update).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = try_main().await {
        eprintln!("Error: {}!", err);
    }
}
