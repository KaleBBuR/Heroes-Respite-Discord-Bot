pub mod db;
pub mod party_groups;

use std::{collections::HashSet, convert::TryFrom, env, time::Duration, thread};
use serenity::{
    async_trait,
    builder::CreateEmbedAuthor,
    client::Client,
    framework::standard::{
        Args, CommandResult, StandardFramework, CommandError
        macros::*,
    },
    futures::StreamExt,
    http::Http,
    model::prelude::*,
    prelude::*,
    utils::Colour
};
use mongodb::{Client as ClientDB, options::ClientOptions};
use db::Database;
use crate::db::DatabaseServer;
use crate::party_groups::Group;

/*
 * Thank you Kara-b
 * https://github.com/kara-b/kbot_rust/tree/01bbbec4c1ce6497e58141e0495441c5f446bd18
 */

const THUMBS_UP: &str = "👍";

struct Handler;

#[group]
#[description = "A group of commands that allow you to create guild parties!"]
#[prefixes("party", "p")]
#[only_in(guilds)]
#[commands(create)]
struct Party;

#[group]
#[description = "Commands only the owner can use to help the bot."]
#[prefixes("owner", "own")]
#[commands(stop)]
#[owners_only]
struct Owner;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn guild_delete(&self, _ctx: Context, _incomplete: GuildUnavailable) {
        let id = _incomplete.id.0;
        match DatabaseServer::delete(&_ctx, id as i64).await {
            Ok(_) => {},
            Err(why) => panic!("Error Deleting from Database\nReason: {}", why)
        };
    }

    async fn guild_create(&self, _ctx: Context, _guild: Guild) {
        let id = _guild.id.0;
        let owner_id = _guild.owner_id.0;
        DatabaseServer::get_or_insert_new(
            &_ctx,
            id as i64,
            Some(owner_id as i64)
        ).await;
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        eprint!("\nNAME: {} is connected!\nID: {}\n", ready.user.name, ready.user.id);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }

            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why)
            }
        },
        Err(why) => panic!("Could not access the bot id: {:?}", why)
    };

    // Our Command Framework
    let framework = StandardFramework::new()
        .configure(|c| c
                    .with_whitespace(true)
                    .on_mention(Some(bot_id))
                    .prefix(">?")
                    .allow_dm(false)
                    .ignore_bots(true)
                    .owners(owners))
    // Set a function to be called prior to each command execution. This
    // provides the context of the command, the message that was received,
    // and the full name of the command that will be called.
    //
    // You can not use this to determine whether a command should be
    // executed. Instead, the `#[check]` macro gives you this functionality.
    //
    // **Note**: Async closures are unstable, you may use them in your
    // application if you are fine using nightly Rust.
    // If not, we need to provide the function identifiers to the
    // hook-functions (before, after, normal, ...).
        // TODO
        // .before(before)
    // Similar to `before`, except will be called directly _after_
    // command execution.
        // TODO
        // .after(after)
    // Set a function that's called whenever an attempted command-call's
    // command could not be found.
        // TODO
        // .unrecognised_command(unknown_command)
    // Set a function that's called whenever a message is not a command.
        // TODO
        // .normal_message(normal_message)
    // Set a function that's called whenever a command's execution didn't complete for one
    // reason or another. For example, when a user has exceeded a rate-limit or a command
    // can only be performed by the bot owner.
        // TODO
        // .on_dispact_error(dispach_error)
        .group(&PARTY_GROUP);

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut bot_client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mongo_url = env::var("MONGO_URL").expect("Expected MongoDB Connection String.");

        // Parse a connection string into an options struct.
        let mut db_client_ops = ClientOptions::parse(mongo_url.as_str())
            .await
            .expect("Could not parse");
        // Manually set an option.
        db_client_ops.app_name = Some("hr_rust_bot".to_string());

        // Get a handle to the deployment
        let client = ClientDB::with_options(db_client_ops).expect("Could not connect to DB");
        let mut data = bot_client.data.write().await;
        data.insert::<Database>(client);
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = bot_client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
#[aliases(c)]
#[required_permissions("ADMINISTRATOR")] // Will be changed.
// It asks for the amount of players and and title of the lobby.
// It would make the title of the lobby the voice chat.
// So it would create a new role referencing the private party.
// So you would have to react with the emoji under it to get the role to access the private voice
// chat for the party.
// I can make it so people can't react to it anymore after the specified amount of players
async fn create(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let player_amount = args.single::<f64>()? as u32;
    let game = args.single::<String>()?;
    let title = String::from(args.rest());

    let guild = msg.guild_id.unwrap();
    let channel = msg.channel_id;

    let author = &msg.author;

    if DatabaseServer::party_owner(ctx, guild.0 as i64, author.id.0 as i64).await {
        let msg = channel.send_message(&ctx.http, |cm| {
            cm.embed(|ce| {
                ce.title("You already have a party created!");
                ce.colour(Colour::RED);
                ce
            });
            cm
        }).await?;

        thread::sleep(Duration::from_secs(20));
        msg.delete(&ctx.http).await?;

        return Ok(())
    }

    let avatar_url: String = match author.avatar_url() {
        Some(url) => url,
        None => {
            let bot_info = match ctx.http.get_current_user().await {
                Ok(bot_info) => bot_info,
                Err(why) => panic!("Could not get bot info: {}", why)
            };

            bot_info.avatar_url().unwrap()
        }
    };

    let party_role = guild.create_role(&ctx.http, |er| {
        er.name(format!("Party Group: {}", author.name))
            .mentionable(true)
    }).await?;

    if player_amount < 20 {
        let party_role_id = party_role.id;
        let everyone_id = RoleId::from(guild.0);

        let mut allow = Permissions::empty();
        allow.insert(Permissions::READ_MESSAGES);
        allow.insert(Permissions::SEND_MESSAGES);

        let perms = vec![
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::all(),
                kind: PermissionOverwriteType::Role(everyone_id),
            },
            PermissionOverwrite {
                allow: allow,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(party_role_id),
            }
        ];

        let party_text_channel = guild.create_channel(&ctx.http, |cc| {
                cc.name(&title)
                    .kind(ChannelType::Text)
                    .permissions(perms.clone())
                    .topic(format!("A Group Party created by: {}", ))
        }).await?;

        let party_text_id = party_text_channel.id;
        let party_voice_channel = guild.create_channel(&ctx.http, |cc| {
            cc.name(&title)
                .kind(ChannelType::Voice)
                .user_limit(player_amount)
                .permissions(perms.clone())
        }).await?;

        let party_voice_id = party_voice_channel.id;
        let mut embed_message = channel.send_message(&ctx.http, |cm| {
            cm.embed(|ce| {
                let mut author_embed = CreateEmbedAuthor::default();
                author_embed.icon_url(&avatar_url);
                author_embed.name(&title);
                let desc = format!("This is a party created by {}", author.name);
                ce.description(desc);
                ce.set_author(author_embed);
                ce.thumbnail(&avatar_url);
                ce.colour(Colour::DARK_GOLD);
                ce.field("Players", "None", true);
                ce
            });
            cm
        }).await?;

        embed_message.react(
            &ctx.http,
            ReactionType::try_from(THUMBS_UP).unwrap()
        ).await?;

        let mut group_data = Group::new(
            msg.author.id.0 as i64,
            player_amount as i64,
            title.clone(),
            game,
            party_voice_id.0 as i64,
            party_text_id.0 as i64,
            party_role_id.0 as i64
        ).await;

        let mut server_data = DatabaseServer::get_or_insert_new(ctx, guild.0 as i64, None).await;
        server_data.add_party(group_data.clone()).await;

        let mut add_reac_collector = embed_message
            .await_reactions(&ctx)
            .removed(true)
            .await;

        while let Some(action) = add_reac_collector.next().await {
            let user_id = &action
                .as_inner_ref()
                .user_id
                .unwrap();
            let user = user_id.to_user(&ctx.http).await?;
            let id = user_id.0;
            let emoji = &action.as_inner_ref().emoji;

            if group_data.full() {
                &ctx.http.delete_reaction(
                    channel.0,
                    msg.id.0,
                    Some(user.id.0),
                    emoji
                ).await?;

                continue
            }

            match emoji.as_data().as_str() {
                THUMBS_UP => {
                    if action.is_added() && !group_data.in_player_vec(&(id as i64)) {
                        group_data.add_player(id as i64).await;
                        group_data.add_player_name(user.name.clone()).await;
                        &ctx.http.add_member_role(
                            guild.0,
                            id as u64,
                            party_role_id.0
                        );
                        let mut member = guild.member(
                            &ctx.http,
                            id as u64
                        ).await?;
                        member.add_role(&ctx.http, party_role_id).await?;
                        embed_message.edit(&ctx.http, |em| {
                            em.embed(|ce| {
                                let mut author_embed = CreateEmbedAuthor::default();
                                author_embed.icon_url(&avatar_url);
                                author_embed.name(&title);
                                let desc = format!(
                                    "This is a party created by {}", author.name
                                );
                                ce.description(desc);
                                ce.set_author(author_embed);
                                ce.thumbnail(&avatar_url);
                                ce.colour(Colour::DARK_GOLD);
                                ce.field(
                                    "Players",
                                    group_data.players(),
                                    true
                                );
                                ce
                            });
                            em
                        }).await?;
                    } else if action.is_removed() && group_data.in_player_vec(&(id as i64)) {
                        group_data.remove_player(id as i64).await;
                        group_data.remove_player_name(user.name.clone()).await;
                        &ctx.http.remove_member_role(
                            guild.0,
                            id as u64,
                            party_role_id.0
                        );
                        let mut member = guild.member(
                            &ctx.http,
                            id as u64
                        ).await?;
                        member.remove_role(&ctx.http, party_role_id).await?;
                        embed_message.edit(&ctx.http, |em| {
                            em.embed(|ce| {
                                let mut author_embed = CreateEmbedAuthor::default();
                                author_embed.icon_url(&avatar_url);
                                author_embed.name(&title);
                                let desc = format!(
                                    "This is a party created by {}", author.name
                                );
                                ce.description(desc);
                                ce.set_author(author_embed);
                                ce.thumbnail(&avatar_url);
                                ce.colour(Colour::DARK_GOLD);
                                ce.field(
                                    "Players",
                                    group_data.players(),
                                    true
                                );
                                ce
                            });
                            em
                        }).await?;
                    }
                },
                _ => {
                    &ctx.http.delete_reaction(
                        channel.0,
                        msg.id.0,
                        Some(user.id.0),
                        emoji
                    ).await?;
                }
            }
        }
    } else {
        let msg = channel.send_message(&ctx.http, |cm| {
            cm.embed(|ce| {
                ce.title("Player limit must be under 20 players!");
                ce.colour(Colour::RED);
                ce
            });
            cm
        }).await?;

        thread::sleep(Duration::from_secs(20));
        msg.delete(&ctx.http).await?;
    }

    Ok(())
}

#[command]
async fn stop(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    unimplemented!()
}