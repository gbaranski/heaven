use crate::database::Database;
use crate::models::MinecraftType;
use crate::models::User;
use crate::models::UserID;
use serenity::async_trait;
use serenity::builder::{CreateActionRow, CreateButton, CreateSelectMenu, CreateSelectMenuOption};
use serenity::client::{Context, EventHandler};
use serenity::futures::StreamExt;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::component::InputTextStyle;
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::channel::Message;
use serenity::model::prelude::component::ActionRowComponent;
use serenity::model::prelude::{Ready, ResumedEvent};
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

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

#[derive(Debug)]
enum Animal {
    Cat,
    Dog,
    Horse,
    Alpaca,
}

#[derive(Debug)]
struct ParseComponentError(String);

impl fmt::Display for ParseComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse {} as component", self.0)
    }
}

impl StdError for ParseComponentError {}

impl fmt::Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cat => write!(f, "Cat"),
            Self::Dog => write!(f, "Dog"),
            Self::Horse => write!(f, "Horse"),
            Self::Alpaca => write!(f, "Alpaca"),
        }
    }
}

impl FromStr for Animal {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cat" => Ok(Animal::Cat),
            "dog" => Ok(Animal::Dog),
            "horse" => Ok(Animal::Horse),
            "alpaca" => Ok(Animal::Alpaca),
            _ => Err(ParseComponentError(s.to_string())),
        }
    }
}

impl Animal {
    fn emoji(&self) -> char {
        match self {
            Self::Cat => '🐈',
            Self::Dog => '🐕',
            Self::Horse => '🐎',
            Self::Alpaca => '🦙',
        }
    }

    fn menu_option(&self) -> CreateSelectMenuOption {
        let mut opt = CreateSelectMenuOption::default();
        // This is what will be shown to the user
        opt.label(format!("{} {}", self.emoji(), self));
        // This is used to identify the selected value
        opt.value(self.to_string().to_ascii_lowercase());
        opt
    }

    fn select_menu() -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id("animal_select");
        menu.placeholder("No animal selected");
        menu.options(|f| {
            f.add_option(Self::Cat.menu_option())
                .add_option(Self::Dog.menu_option())
                .add_option(Self::Horse.menu_option())
                .add_option(Self::Alpaca.menu_option())
        });
        menu
    }

    fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        // A select menu must be the only thing in an action row!
        ar.add_select_menu(Self::select_menu());
        ar
    }
}

#[derive(Debug)]
enum Sound {
    Meow,
    Woof,
    Neigh,
    Honk,
}

impl fmt::Display for Sound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Meow => write!(f, "meow"),
            Self::Woof => write!(f, "woof"),
            Self::Neigh => write!(f, "neigh"),
            Self::Honk => write!(f, "hoooooooonk"),
        }
    }
}

impl Sound {
    fn emoji(&self) -> char {
        match self {
            Self::Meow => Animal::Cat.emoji(),
            Self::Woof => Animal::Dog.emoji(),
            Self::Neigh => Animal::Horse.emoji(),
            Self::Honk => Animal::Alpaca.emoji(),
        }
    }

    fn button(&self) -> CreateButton {
        let mut b = CreateButton::default();
        b.custom_id(self.to_string().to_ascii_lowercase());
        b.emoji(self.emoji());
        b.label(self);
        b.style(ButtonStyle::Primary);
        b
    }

    fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        // We can add up to 5 buttons per action row
        ar.add_button(Sound::Meow.button());
        ar.add_button(Sound::Woof.button());
        ar.add_button(Sound::Neigh.button());
        ar.add_button(Sound::Honk.button());
        ar
    }
}

impl FromStr for Sound {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "meow" => Ok(Sound::Meow),
            "woof" => Ok(Sound::Woof),
            "neigh" => Ok(Sound::Neigh),
            "hoooooooonk" => Ok(Sound::Honk),
            _ => Err(ParseComponentError(s.to_string())),
        }
    }
}

pub struct Bot {
    database: Database,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        use serenity::model::application::command::Command;

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("announce")
                .description("Announce a message to the channel")
        })
        .await;

        tracing::info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        tracing::info!("Resumed");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => match command.data.name.as_str() {
                "announce" => {
                    command
                        .channel_id
                        .send_message(&ctx, |m| {
                            m.content("Hello!")
                                .components(|c| c.add_action_row(action_row()))
                        })
                        .await
                        .unwrap();
                    command
                        .create_interaction_response(&ctx, |f| {
                            f.interaction_response_data(|f| {
                                f.ephemeral(true)
                                    .content(format!("Send annoucment onto {}", command.channel_id))
                            })
                        })
                        .await
                        .unwrap();
                }
                other => {
                    tracing::error!("unknown command name: {other}");
                }
            },
            Interaction::MessageComponent(mc) => match mc.data.custom_id.as_str() {
                "register" => {
                    if let Some(user) = self.database.get_user_by_discord_id(mc.user.id) {
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
                other => {
                    tracing::error!("unknown message component id: {other}");
                }
            },
            Interaction::ModalSubmit(submission) => match submission.data.custom_id.as_str() {
                "registration" => {
                    if let Some(user) = self.database.get_user_by_discord_id(submission.user.id) {
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

                    if let Some(user) = self.database.get_user_by_minecraft_name(&minecraft_name) {
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
                    let user = User {
                        id: UserID::new_v4(),
                        discord_id: submission.user.id,
                        discord_name: submission
                            .user
                            .nick_in(&ctx, submission.guild_id.unwrap())
                            .await
                            .unwrap_or_else(|| submission.user.name.clone()),
                        minecraft_type,
                        minecraft_name,
                    };
                    self.database.insert_user(&user);
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

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content != "animal" {
            return;
        }

        // Ask the user for its favorite animal
        let m = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.content("Please select your favorite animal")
                    .components(|c| c.add_action_row(Animal::action_row()))
            })
            .await
            .unwrap();

        // Wait for the user to make a selection
        let mci = match m
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(60 * 3))
            .await
        {
            Some(ci) => ci,
            None => {
                m.reply(&ctx, "Timed out").await.unwrap();
                return;
            }
        };

        // data.custom_id contains the id of the component (here "animal_select")
        // and should be used to identify if a message has multiple components.
        // data.values contains the selected values from the menu
        let animal = Animal::from_str(mci.data.values.get(0).unwrap()).unwrap();

        // Acknowledge the interaction and edit the message
        mci.create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.content(format!("You chose: **{}**\nNow choose a sound!", animal))
                        .components(|c| c.add_action_row(Sound::action_row()))
                })
        })
        .await
        .unwrap();

        // Wait for multiple interactions

        let mut cib = m
            .await_component_interactions(&ctx)
            .timeout(Duration::from_secs(60 * 3))
            .build();

        while let Some(mci) = cib.next().await {
            let sound = Sound::from_str(&mci.data.custom_id).unwrap();
            // Acknowledge the interaction and send a reply
            mci.create_interaction_response(&ctx, |r| {
                // This time we dont edit the message but reply to it
                r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        // Make the message hidden for other users by setting `ephemeral(true)`.
                        d.ephemeral(true)
                            .content(format!("The **{}** says __{}__", animal, sound))
                    })
            })
            .await
            .unwrap();
        }

        // Delete the orig message or there will be dangling components
        m.delete(&ctx).await.unwrap()
    }
}

impl Bot {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}
