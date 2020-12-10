pub mod db;
pub mod party_groups;

use std::{collections::HashSet, convert::TryFrom, env, time::Duration, thread};
use serenity::{
    async_trait,
    builder::CreateEmbedAuthor,
    client::Client,
    collector::ReactionAction,
    framework::standard::{
        Args, CommandResult, StandardFramework,
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

    async fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {
        todo!()
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        eprint!("\nNAME: {} is connected!\nID: {}", ready.user.name, ready.user.id);
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
                    .delimiters(vec![", ", ","])
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
// It asks for the amount of players and and title of the lobby.
// It would make the title of the lobby the voice chat.
// So it would create a new role referencing the private party.
// So you would have to react with the emoji under it to get the role to access the private voice
// chat for the party.
// I can make it so people can't react to it anymore after the specified amount of players
async fn create(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let title = args.single::<String>()?;
    let player_amount = args.single::<u32>()?;
    let game = args.single::<String>()?;

    let guild = msg.guild_id.unwrap();
    let party_role = guild.create_role(ctx.http.clone(), |er| {
        er.name(title.clone())
            .mentionable(true)
    }).await;

    if player_amount < 20 {
        match party_role {
            Ok(role) => {
                let party_role_id = role.id;
                let party_channel_permissions = Some(PermissionOverwrite {
                    allow: Permissions::READ_MESSAGES,
                    deny: Permissions::SEND_TTS_MESSAGES,
                    kind: PermissionOverwriteType::Role(party_role_id)
                });

                let party_text_channel = guild.create_channel(&ctx.http, |cc| {
                        cc.name(&title)
                            .kind(ChannelType::Text)
                            .permissions(party_channel_permissions.clone())
                            .topic(format!("A party created for: {}", game))
                }).await;

                match party_text_channel {
                    Ok(guild_text_channel) => {
                        let guild_text_channel_id = guild_text_channel.id;
                        let party_text_channel = guild.create_channel(&ctx.http, |cc| {
                            cc.name(&title)
                                .kind(ChannelType::Voice)
                                .user_limit(player_amount)
                                .permissions(party_channel_permissions.clone())
                        }).await;

                        match party_text_channel {
                            Ok(guild_voice_channel) => {
                                let guild_voice_channel_id = guild_voice_channel.id;
                                let embed_message = msg.channel_id.send_message(&ctx.http, |cm| {
                                    cm.embed(|ce| {
                                        let mut author = CreateEmbedAuthor::default();
                                        let avatar_url = msg.author.avatar_url().unwrap();
                                        author.icon_url(&avatar_url);
                                        author.name(&title);
                                        ce.description(
                                            format!("This is a party created for! -> {}", game)
                                        );
                                        ce.set_author(author);
                                        ce.thumbnail(&avatar_url);
                                        ce.colour(Colour::DARK_ORANGE);
                                        ce.field("Playes", "", true);
                                        ce
                                    });
                                    cm
                                }).await;
                                match embed_message {
                                    Ok(msg) => {
                                        msg.react(
                                            &ctx.http,
                                            ReactionType::try_from(THUMBS_UP).unwrap()
                                        ).await.unwrap();

                                        let mut party_group_struct = Group::new(
                                            msg.author.id.0 as usize,
                                            player_amount as usize,
                                            title,
                                            game
                                        );

                                        let collector = &mut msg
                                            .await_reactions(&ctx)
                                            .timeout(Duration::from_secs(1))
                                            .await;

                                        while let Some(action) = collector.next().await {
                                            let user_id = &action
                                                .as_inner_ref()
                                                .user_id
                                                .unwrap();
                                            let user = user_id.to_user(&ctx.http).await;
                                            let id = user_id.0 as usize;
                                            match user {
                                                Ok(_) => {
                                                    let emoji = &action.as_inner_ref().emoji;
                                                    match emoji.as_data().as_str() {
                                                        THUMBS_UP => {
                                                            if !party_group_struct
                                                            .in_player_vec(&id)
                                                            {
                                                                party_group_struct
                                                                .add_player(id);
                                                            }
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                Err(why) => {
                                                    return_error_embed(ctx, &msg, why).await?;
                                                }
                                            }
                                        }
                                    },
                                    Err(why) => {
                                        return_error_embed(ctx, &msg, why).await?
                                    }
                                }
                            },
                            Err(why) => {
                                return_error_embed(ctx, &msg, why).await?
                            }
                        }
                    },
                    Err(why) => {
                        return_error_embed(ctx, &msg, why).await?
                    }
                }
            },
            Err(why) => {
                return_error_embed(ctx, &msg, why).await?
            }
        };
    } else {
        msg.channel_id.send_message(&ctx.http, |cm| {
            cm.content("The player limit must be under 20!");
            cm
        }).await?;
    }

    Ok(())
}

async fn return_error_embed(ctx: &Context, msg: &Message, error: serenity::Error) -> CommandResult {
    let mut count = 0;
    loop {
        if count == 3 { break }
        let bot_info = &ctx.http.get_current_application_info().await;
        match bot_info {
            Ok(info) => {
                let bot_id = info.id.0;
                let avatar_url = &ctx.http.get_user(bot_id).await.unwrap().avatar_url().unwrap();
                let _ = msg.channel_id.send_message(&ctx.http, |cm| {
                    cm.embed(|ce| {
                        let mut author = CreateEmbedAuthor::default();
                        author.icon_url(avatar_url);
                        author.name("ERROR ERROR");
                        ce.set_author(author);
                        ce.thumbnail(&avatar_url);
                        ce.colour(Colour::RED);
                        ce.field("Error", error.to_string(), true);
                        ce
                    });
                    cm
                }).await?;
                break
            },
            Err(_) => {
                let time = Duration::from_secs(1);
                thread::sleep(time);
                count += 1;
                continue;
            }
        };
    }

    Ok(())
}