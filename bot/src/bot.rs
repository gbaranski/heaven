use crate::database::Database;
use crate::models::Angel;
use crate::models::AngelID;
use crate::models::MinecraftType;
use crate::store::Store;
use serenity::async_trait;
use serenity::builder::{CreateActionRow, CreateButton};
use serenity::client::{Context, EventHandler};
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::component::InputTextStyle;
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::component::ActionRowComponent;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::{Ready, ResumedEvent};

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

pub struct Bot {
    whitelist_channel_id: ChannelId,
    database: Database,
    store: Store,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let previous_messages = self
            .whitelist_channel_id
            .messages(&ctx, |m| m)
            .await
            .unwrap();
        let message_ids = previous_messages
            .iter()
            .filter(|m| m.author.id == ready.user.id)
            .map(|m| m.id);
        self.whitelist_channel_id
            .delete_messages(&ctx, message_ids)
            .await
            .unwrap();
        self.whitelist_channel_id
            .send_message(&ctx, |m| {
                m.content("Hello! Tap the button below to register your Minecraft account within the server.")
                    .components(|c| c.add_action_row(action_row()))
            })
            .await
            .unwrap();

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
                                    "You are already registered under {} name!",
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
                    let success = self.store.allow(mc.user.id);
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
                    let success = self.store.deny(mc.user.id);
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

impl Bot {
    pub fn new(database: Database, store: Store, whitelist_channel_id: ChannelId) -> Self {
        Self {
            database,
            store,
            whitelist_channel_id,
        }
    }
}
