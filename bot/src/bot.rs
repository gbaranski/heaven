use std::net::IpAddr;
use std::sync::Arc;

use crate::authorizations::Authorization;
use crate::authorizations::Authorizations;
use crate::configuration::Configuration;
use crate::database::Database;
use crate::models::Angel;
use crate::models::AngelID;
use crate::models::MinecraftType;
use crate::ShardManagerContainer;
use miette::{IntoDiagnostic, Result};
use serenity::async_trait;
use serenity::builder::{CreateActionRow, CreateButton};
use serenity::client::{Context, EventHandler};
use serenity::http::Http;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::component::InputTextStyle;
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::component::ActionRowComponent;
use serenity::model::prelude::Activity;
use serenity::model::prelude::UserId;
use serenity::model::prelude::{Ready, ResumedEvent};
use serenity::prelude::GatewayIntents;
use serenity::Client;
use tokio_graceful_shutdown::SubsystemHandle;

const BOT_OWNER_ID: &str = "398874695069335571";

fn register_button() -> CreateButton {
    let mut b = CreateButton::default();
    b.custom_id("register");
    b.label("Register");
    b.style(ButtonStyle::Primary);
    b
}

fn action_row() -> CreateActionRow {
    let mut ar = CreateActionRow::default();
    // We can add up to 5 buttons per action row
    ar.add_button(register_button());
    ar
}

#[derive(Debug, Clone)]
pub struct DiscordBot {
    configuration: Arc<Configuration>,
    database: Database,
    authorizations: Authorizations,
    client: Arc<Http>,
}

#[async_trait]
impl EventHandler for DiscordBot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let mut activity = Activity::watching("till gbaranski gets his ban removed".to_string());
        activity.details = Some("Check out my GitHub profile https://github.com/gbaranski/heaven".to_string());
        ctx.set_activity(activity).await;
        let greeting_content =
            format!("Hello! Tap the button below to register your Discord account within the Minecraft Server.\n> *Bot created by <@{BOT_OWNER_ID}>*");
        let previous_messages = self
            .configuration
            .whitelist_channel_id
            .messages(&ctx, |m| m)
            .await
            .unwrap();
        let mut previous_messages = previous_messages
            .into_iter()
            .filter(|m| m.author.id == ready.user.id)
            .collect::<Vec<_>>();

        if let Some((last_message, previous_messages)) = previous_messages.split_last_mut() {
            if previous_messages.len() > 0 {
                self.configuration
                    .whitelist_channel_id
                    .delete_messages(&ctx, previous_messages)
                    .await
                    .unwrap();
            }
            last_message
                .edit(&ctx, |m| {
                    m.content(greeting_content)
                        .components(|c| c.add_action_row(action_row()))
                })
                .await
                .unwrap();
        } else {
            self.configuration
                .whitelist_channel_id
                .send_message(&ctx, |m| {
                    m.content(greeting_content)
                        .components(|c| c.add_action_row(action_row()))
                })
                .await
                .unwrap();
        }
        tracing::info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        tracing::info!("Resumed");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => match command.data.name.as_str() {
                other => {
                    tracing::error!("unknown command name: {other}");
                }
            },
            Interaction::MessageComponent(mc) => match mc.data.custom_id.as_str() {
                "register" => {
                    if let Some(user) = self.database.get_angel_by_discord_id(mc.user.id) {
                        mc.create_interaction_response(&ctx, |i| {
                            i.interaction_response_data(|d| {
                                d.ephemeral(true).content(format!(
                                    "You are already registered under name {}! If that's not your minecraft nickname, please report to <@{BOT_OWNER_ID}>",
                                    user.minecraft_name
                                ))
                            })
                        })
                        .await
                        .unwrap();
                        return;
                    }
                    mc.create_interaction_response(&ctx, |i| {
                        i.kind(InteractionResponseType::Modal)
                            .interaction_response_data(|m| {
                                m.components(|c| {
                                    c.create_action_row(|ar| {
                                        ar.create_input_text(|it| {
                                            it.placeholder("Enter your Minecraft account-name")
                                                .label("Minecraft name")
                                                .custom_id("minecraft-name")
                                                .required(true)
                                                .style(InputTextStyle::Short)
                                        });
                                        ar
                                    });
                                    c.create_action_row(|ar| {
                                        ar.create_select_menu(|sm| {
                                            sm.placeholder("Type of Minecraft account")
                                                .custom_id("minecraft-type")
                                                .options(|o| {
                                                    o.create_option(|o| {
                                                        o.value("premium").label("Premium")
                                                    });
                                                    o.create_option(|o| {
                                                        o.value("Cracked").label("Cracked")
                                                    });
                                                    o
                                                })
                                        })
                                    });
                                    c
                                })
                                .title("Minecraft user registration")
                                .custom_id("registration")
                            })
                    })
                    .await
                    .unwrap();
                }
                "authorization/allow" => {
                    let success = self.authorizations.allow(mc.user.id);
                    let message = if success {
                        "Authorization allowed! ✅"
                    } else {
                        "Error: Authorization expired or already allowed!"
                    };
                    mc.create_interaction_response(&ctx, |i| {
                        i.interaction_response_data(|d| d.content(message))
                    })
                    .await
                    .unwrap();
                }
                "authorization/deny" => {
                    let success = self.authorizations.deny(mc.user.id);
                    let message = if success {
                        "Authorization denied! ❌"
                    } else {
                        "Error: Authorization expired or already denied!"
                    };
                    mc.create_interaction_response(&ctx, |i| {
                        i.interaction_response_data(|d| d.content(message))
                    })
                    .await
                    .unwrap();
                }
                other => {
                    tracing::error!("unknown message component id: {other}");
                }
            },
            Interaction::ModalSubmit(submission) => match submission.data.custom_id.as_str() {
                "registration" => {
                    if let Some(user) = self.database.get_angel_by_discord_id(submission.user.id) {
                        submission
                            .create_interaction_response(&ctx, |i| {
                                i.interaction_response_data(|d| {
                                    d.ephemeral(true).content(format!(
                                        "You are already registered under {} name!",
                                        user.minecraft_name
                                    ))
                                })
                            })
                            .await
                            .unwrap();
                        return;
                    }
                    let minecraft_name = if let ActionRowComponent::InputText(component) =
                        &submission.data.components[0].components[0]
                    {
                        assert_eq!(component.custom_id, "minecraft-name");
                        component.value.clone()
                    } else {
                        panic!("invalid component type");
                    };
                    let minecraft_type = if let ActionRowComponent::SelectMenu(component) =
                        &submission.data.components[1].components[0]
                    {
                        assert_eq!(component.custom_id.as_ref().unwrap(), "minecraft-type");
                        assert_eq!(component.values.len(), 1);
                        match component.values[0].as_str() {
                            "premium" => MinecraftType::Premium,
                            "cracked" => MinecraftType::Cracked,
                            other => panic!("invalid account type: {other}"),
                        }
                    } else {
                        panic!("invalid component type");
                    };

                    if let Some(user) = self.database.get_angel_by_minecraft_name(&minecraft_name) {
                        submission
                            .create_interaction_response(&ctx, |i| {
                                i.interaction_response_data(|d| {
                                    d.ephemeral(true).content(format!(
                                        "Someone's already registered under {} name!",
                                        user.minecraft_name
                                    ))
                                })
                            })
                            .await
                            .unwrap();
                        return;
                    }
                    let angel = Angel {
                        id: AngelID::new_v4(),
                        discord_id: submission.user.id,
                        discord_name: submission
                            .user
                            .nick_in(&ctx, submission.guild_id.unwrap())
                            .await
                            .unwrap_or_else(|| submission.user.name.clone()),
                        minecraft_type,
                        minecraft_name,
                    };
                    self.database.insert_angel(&angel);
                    submission.create_interaction_response(&ctx, |i| {
                        i.interaction_response_data(|d| {
                            d.ephemeral(true).content("Thanks for registering! You should be able to log in into your Minecraft account now.")
                        })
                    }).await.unwrap();
                }
                other => {
                    tracing::error!("unknown modal submission id: {other}");
                }
            },
            _ => {}
        }
    }
}

impl DiscordBot {
    pub fn new(configuration: Arc<Configuration>, database: Database) -> Self {
        let http = Http::new(&configuration.discord_token);
        let authorizations = Authorizations::new();

        Self {
            client: Arc::new(http),
            database,
            authorizations,
            configuration,
        }
    }

    pub async fn run(self, subsystem: SubsystemHandle) -> Result<()> {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;
        let mut client = Client::builder(self.configuration.discord_token.clone(), intents)
            .event_handler(self)
            .await
            .expect("Error creating client");
        {
            let mut data = client.data.write().await;
            data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        }
        let shard_manager = client.shard_manager.clone();
        tokio::select! {
            result = client.start() => {
                result.into_diagnostic()
            }
            _ = subsystem.on_shutdown_requested() => {
                tracing::info!("Shutting down Discord Bot");
                shard_manager.lock().await.shutdown_all().await;
                Ok(())
            }
        }
    }

    pub async fn authorize(&self, user_id: UserId, from: IpAddr) -> Result<Authorization> {
        let user = self.client.get_user(user_id.0).await.unwrap();
        let dm_channel = user.create_dm_channel(&self.client).await.unwrap();
        let authorization = self.authorizations.request_authorization(user_id);
        let mut message = dm_channel
            .send_message(&self.client, |f| {
                f.components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_button(|b| {
                            b.custom_id("authorization/allow")
                                .emoji('✅')
                                .label("Allow")
                                .style(ButtonStyle::Primary)
                        })
                        .create_button(|b| {
                            b.custom_id("authorization/deny")
                                .emoji('❌')
                                .label("Deny")
                                .style(ButtonStyle::Secondary)
                        })
                    })
                })
                .content(format!(
                    "New login request for Minecraft server from {from}."
                ))
            })
            .await
            .into_diagnostic()?;
        let authorization = authorization.await;
        message
            .edit(&self.client, |f| {
                f.components(|f| f.set_action_rows(vec![]))
            })
            .await
            .into_diagnostic()?;
        Ok(authorization)
    }
}
