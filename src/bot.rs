use crate::{CONFIG, CONFIG_PATH};
use std::sync::Arc;

use regex::Regex;
use serenity::{
    client::{Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::{
        channel::{Channel, Message},
        guild::Member,
        id::{ChannelId, GuildId},
        voice::VoiceState,
    },
};

use lazy_static::lazy_static;

#[group]
#[commands(link)]
struct Commands;

pub struct Handler;

pub async fn parse_link_command(_ctx: &Context, msg: &str) -> Option<(ChannelId, ChannelId)> {
    // Link commands will be of the form
    // |>link <voice channel id> <text channel id>
    lazy_static! {
        // I've commented it down below to make it nicer to look at but just look at this shit
        //
        // ^\|>link\s+(?:<\#)?(\d+)>?\s+(?:<\#)?(\d+)>?$
        //
        // Insane
        // Who came up with this language??
        static ref LINK_RE: Regex = Regex::new(
            r"(?x)
            ^\|>link            # command prefix
            \s+                 # ignore space
            (?:<\#)? (\d+) >?   # match a channel id (potentially inside <# > brackets)
            \s+                 # more space
            (?:<\#)? (\d+) >?   # another channel id
            $"
        )
        .unwrap();
    }

    if let Some(captures) = LINK_RE.captures_iter(msg.trim()).next() {
        let vcid = ChannelId((&captures[1]).parse::<u64>().ok()?);
        let tcid = ChannelId((&captures[2]).parse::<u64>().ok()?);

        // TODO: Verify these are in the guild

        Some((vcid, tcid))
    } else {
        None
    }
}

#[command]
async fn link(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some((vcid, tcid)) = parse_link_command(ctx, &msg.content).await {
        msg.reply(
            Arc::clone(&ctx.http),
            format!("linking <#{}> to <#{}> 👍", vcid.0, tcid.0),
        )
        .await
        .unwrap();

        let mut conf = CONFIG.lock().unwrap();
        conf.channels.insert(vcid, tcid);
        conf.save_to_file(CONFIG_PATH)?;
        drop(conf);

        log::info!("command called: \"{}\"", msg.content);
    }

    Ok(())
}

async fn check_vc_event(
    ctx: &Context,
    old: Option<VoiceState>,
    new: VoiceState,
) -> Option<(Channel, Member)> {
    let old = old?;

    if old.user_id == new.user_id && old.channel_id != new.channel_id {
        let member = old.member?;
        let channel = old
            .channel_id?
            .to_channel(Arc::clone(&ctx.http))
            .await
            .ok()?;

        Some((channel, member))
    } else {
        None
    }
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        if message.content == "%melo" {
            message.react(ctx.http, '🍈').await.unwrap();
        }
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        _gid: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        if let Some((Channel::Guild(channel), member)) = check_vc_event(&ctx, old, new).await {
            log::info!(
                "user {} left channel {}",
                member.display_name(),
                channel.name()
            );

            let res = {
                let conf = CONFIG.lock().unwrap();
                conf.channels.get(&channel.id).map(|cid| cid.to_owned())
            };

            if let Some(tcid) = res {
                if let Err(e) = tcid
                    .say(
                        ctx.http,
                        format!(
                            "{} left voice channel {}",
                            member.display_name(),
                            channel.name()
                        ),
                    )
                    .await
                {
                    log::error!("Error sending message: {e:?}");
                }
            }
        }
    }
}
