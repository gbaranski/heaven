package main

import (
	"fmt"
	"strings"

	"github.com/bwmarrin/discordgo"
	"github.com/google/uuid"
	"github.com/rs/zerolog/log"
)

var session *discordgo.Session
var authorizations map[string]chan bool

func InitBot(token string) error {
	authorizations = make(map[string]chan bool)
	var err error
	session, err = discordgo.New("Bot " + token)
	if err != nil {
		return err
	}

	session.AddHandler(func(s *discordgo.Session, r *discordgo.Ready) {
		log.Info().
			Str("name", fmt.Sprintf("%v#%v", s.State.User.Username, s.State.User.Discriminator)).
			Msg("Logged in")
	})
	session.AddHandler(func(s *discordgo.Session, i *discordgo.InteractionCreate) {
		if i.Type == discordgo.InteractionApplicationCommand {
			if h, ok := commandHandlers[i.ApplicationCommandData().Name]; ok {
				h(s, i)
			} else {
				log.Error().Msgf("no handler for message component interaction %s", i.MessageComponentData().CustomID)
			}
		} else if i.Type == discordgo.InteractionMessageComponent {
			var h func(s *discordgo.Session, i *discordgo.InteractionCreate)
			for name, handler := range messageInteractionHandlers {
				if strings.HasPrefix(i.MessageComponentData().CustomID, name) {
					h = handler
				}
			}
			if h != nil {
				h(s, i)
			} else {
				log.Error().Msgf("no handler for message interaction %s", i.MessageComponentData().CustomID)
			}
		} else if i.Type == discordgo.InteractionModalSubmit {
			var h func(s *discordgo.Session, i *discordgo.InteractionCreate)
			for name, handler := range modalSubmitHandlers {
				if strings.HasPrefix(i.ModalSubmitData().CustomID, name) {
					h = handler
				}
			}

			if h != nil {
				h(s, i)
			} else {
				log.Error().Msgf("no handler for modal submit %s", i.ModalSubmitData().CustomID)
			}
		} else {
			log.Warn().
				Str("type", i.Type.String()).
				Msg("unknown interaction type")
		}
	})

	err = session.Open()
	if err != nil {
		return err
	}

	log.Info().Msg("Adding commands...")
	registeredCommands := make([]*discordgo.ApplicationCommand, len(commands))
	for i, v := range commands {
		cmd, err := session.ApplicationCommandCreate(session.State.User.ID, "", v)
		if err != nil {
			return err
		}
		registeredCommands[i] = cmd
	}

	return nil
}

var (
	minAddressLength = 6
	commands         = []*discordgo.ApplicationCommand{
		{
			Name:        "setup",
			Description: "Setup an Minecraft server",
			Type:        discordgo.ChatApplicationCommand,
			Options: []*discordgo.ApplicationCommandOption{
				{
					Type:        discordgo.ApplicationCommandOptionString,
					Name:        "address",
					Description: "Address of the Minecraft server",
					Required:    true,
					MinLength:   &minAddressLength,
				},
			},
		},
		{
			Name:        "announce",
			Description: "Announce an Minecraft server",
			Type:        discordgo.ChatApplicationCommand,
			Options: []*discordgo.ApplicationCommandOption{
				{
					Type:        discordgo.ApplicationCommandOptionString,
					Name:        "address",
					Description: "Address of the Minecraft server",
					Required:    true,
					MinLength:   &minAddressLength,
				},
			},
		},
	}

	modalSubmitHandlers = map[string]func(s *discordgo.Session, i *discordgo.InteractionCreate){
		"registration/": func(s *discordgo.Session, i *discordgo.InteractionCreate) {
			serverID := strings.SplitAfter(i.ModalSubmitData().CustomID, "/")[1]
			server, err := GetServer(serverID)
			if err != nil {
				log.Error().
					Err(err).
					Msg("failed to get server")
				return
			}
			if server == nil {
				log.Error().
					Err(err).
					Msg("server not found")
				return
			}
			minecraftName := i.ModalSubmitData().Components[0].(*discordgo.ActionsRow).Components[0].(*discordgo.TextInput).Value
			alreadyRegistered, err := DoesAngelExistWithUserID(serverID, i.Member.User.ID)
			if err != nil {
				log.Error().
					Err(err).
					Msg("failed to check if user is already registered")
				return
			}
			if alreadyRegistered {
				s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
					Type: discordgo.InteractionResponseChannelMessageWithSource,
					Data: &discordgo.InteractionResponseData{
						Content: "You're already registered.",
						Flags:   discordgo.MessageFlagsEphemeral,
					},
				})
				return
			}

			existsWithNickname, err := DoesAngelExistWithMinecraftName(serverID, minecraftName)
			if err != nil {
				log.Error().
					Err(err).
					Msg("failed to check if minecraft name is already taken")
				return
			}
			if existsWithNickname {
				s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
					Type: discordgo.InteractionResponseChannelMessageWithSource,
					Data: &discordgo.InteractionResponseData{
						Content: "This nickname is already taken.",
						Flags:   discordgo.MessageFlagsEphemeral,
					},
				})
				return
			}

			if err := AddAngel(&Angel{
				Name:          i.Member.Nick,
				UserID:        i.Member.User.ID,
				ServerID:      serverID,
				MinecraftName: minecraftName,
			}); err != nil {
				log.Error().
					Err(err).
					Msg("failed to add angel")
				return
			}

			log.Info().
				Str("minecraft-name", minecraftName).
				Str("name", i.Member.Nick).
				Str("server", serverID).
				Msg("added angel")

			s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Registered! Now try to connect to the server.",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
		},
		"updation/": func(s *discordgo.Session, i *discordgo.InteractionCreate) {
			serverID := strings.SplitAfter(i.ModalSubmitData().CustomID, "/")[1]
			minecraftName := i.ModalSubmitData().Components[0].(*discordgo.ActionsRow).Components[0].(*discordgo.TextInput).Value
			angel, err := GetAngel(serverID, i.Member.User.ID)
			if err != nil {
				log.Error().
					Err(err).
					Msg("failed to get angel")
				return
			}
			if angel == nil {
				s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
					Type: discordgo.InteractionResponseChannelMessageWithSource,
					Data: &discordgo.InteractionResponseData{
						Content: "You're not registered.",
						Flags:   discordgo.MessageFlagsEphemeral,
					},
				})
				return
			}

			existsWithNickname, err := DoesAngelExistWithMinecraftName(serverID, minecraftName)
			if err != nil {
				log.Error().
					Err(err).
					Msg("failed to check if minecraft name is already taken")
				return
			}
			if existsWithNickname {
				s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
					Type: discordgo.InteractionResponseChannelMessageWithSource,
					Data: &discordgo.InteractionResponseData{
						Content: "This nickname is already taken.",
						Flags:   discordgo.MessageFlagsEphemeral,
					},
				})
				return
			}

			angel.MinecraftName = minecraftName
			UpdateAngel(angel)
			log.Info().
				Str("minecraft-name", minecraftName).
				Str("name", i.Member.Nick).
				Str("server", serverID).
				Msg("updated angel")

			s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Updated! Now try to connect to the server with the new nickname.",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
		},
	}

	messageInteractionHandlers = map[string]func(s *discordgo.Session, i *discordgo.InteractionCreate){
		"authorization/": func(s *discordgo.Session, i *discordgo.InteractionCreate) {
			fmt.Printf("receive message interaction handler")
			splits := strings.Split(i.MessageComponentData().CustomID, "/")
			authorizationID := splits[1]
			fmt.Printf("authorization ID: %s\n", authorizationID)
			result := splits[2]
			var reply string
			fmt.Printf("result: %s\n", result)
			if result == "allow" {
				reply = "Authorization allowed! ✅"
				fmt.Printf("sending")
				authorizations[authorizationID] <- true
				fmt.Printf("sent!")
			} else if result == "deny" {
				reply = "Authorization denied! ❌"
				authorizations[authorizationID] <- false
			} else {
				log.Warn().Str("result", result).Msg("unknown result")
				return
			}
			if err := s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: reply,
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			}); err != nil {
				panic(err)
			}
		},
		"register/": func(s *discordgo.Session, i *discordgo.InteractionCreate) {
			serverID := strings.SplitAfter(i.MessageComponentData().CustomID, "/")[1]
			server, err := GetServer(serverID)
			if err != nil {
				log.Error().
					Err(err).
					Msg("failed to get server")
				return
			}
			if server == nil {
				log.Error().
					Err(err).
					Msg("server not found")
				s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
					Type: discordgo.InteractionResponseChannelMessageWithSource,
					Data: &discordgo.InteractionResponseData{
						Content: "Link is not longer valid",
						Flags:   discordgo.MessageFlagsEphemeral,
					},
				})
				return
			}
			alreadyRegistered, err := DoesAngelExistWithUserID(serverID, i.Member.User.ID)
			if err != nil {
				log.Error().
					Err(err).
					Msg("failed to check if user is already registered")
				return
			}

			if alreadyRegistered {
				s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
					Type: discordgo.InteractionResponseChannelMessageWithSource,
					Data: &discordgo.InteractionResponseData{
						Content: "You're already registered.",
						Flags:   discordgo.MessageFlagsEphemeral,
					},
				})
				return
			}

			if err = s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseModal,
				Data: &discordgo.InteractionResponseData{
					CustomID: fmt.Sprintf("registration/%s", serverID),
					Title:    "Account Registration",
					Components: []discordgo.MessageComponent{
						discordgo.ActionsRow{
							Components: []discordgo.MessageComponent{
								discordgo.TextInput{
									CustomID:    "minecraft-name",
									Label:       "Minecraft Name",
									Style:       discordgo.TextInputShort,
									Placeholder: "Enter your minecraft name",
									Required:    true,
									MinLength:   3,
								},
							},
						},
					},
				},
			}); err != nil {
				panic(err)
			}
		},
		"update/": func(s *discordgo.Session, i *discordgo.InteractionCreate) {
			serverID := strings.SplitAfter(i.MessageComponentData().CustomID, "/")[1]
			server, err := GetServer(serverID)
			if err != nil {
				log.Error().
					Err(err).
					Msg("failed to get server")
				return
			}
			if server == nil {
				log.Error().
					Err(err).
					Msg("server not found")
				s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
					Type: discordgo.InteractionResponseChannelMessageWithSource,
					Data: &discordgo.InteractionResponseData{
						Content: "Link is not longer valid",
						Flags:   discordgo.MessageFlagsEphemeral,
					},
				})
				return
			}
			alreadyRegistered, err := DoesAngelExistWithUserID(serverID, i.Member.User.ID)
			if err != nil {
				log.Error().
					Err(err).
					Msg("failed to check if user is already registered")
				return
			}

			if !alreadyRegistered {
				s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
					Type: discordgo.InteractionResponseChannelMessageWithSource,
					Data: &discordgo.InteractionResponseData{
						Content: "You're not registered.",
						Flags:   discordgo.MessageFlagsEphemeral,
					},
				})
				return
			}

			if err = s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseModal,
				Data: &discordgo.InteractionResponseData{
					CustomID: fmt.Sprintf("updation/%s", serverID),
					Title:    "Account Updation",
					Components: []discordgo.MessageComponent{
						discordgo.ActionsRow{
							Components: []discordgo.MessageComponent{
								discordgo.TextInput{
									CustomID:    "minecraft-name",
									Label:       "New Minecraft Name",
									Style:       discordgo.TextInputShort,
									Placeholder: "Enter your minecraft name",
									Required:    true,
									MinLength:   3,
								},
							},
						},
					},
				},
			}); err != nil {
				panic(err)
			}
		},
	}

	commandHandlers = map[string]func(s *discordgo.Session, i *discordgo.InteractionCreate){
		"setup": func(s *discordgo.Session, i *discordgo.InteractionCreate) {
			address := i.ApplicationCommandData().Options[0].StringValue()

			id := uuid.New()
			token := uuid.New()
			AddServer(&Server{
				ID:      id,
				Address: address,
				Token:   token,
				GuildID: i.GuildID,
			})
			log.Info().
				Str("id", id.String()).
				Str("address", address).
				Str("guild", i.GuildID).
				Msg("adding server")

			s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: fmt.Sprintf("Server has been set up.\nID: `%s`\nToken ||`%s`||\nCopy the ID and Token and paste it into plugin configuration. \n**Do not share the token**", id, token),
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
		},
		"announce": func(s *discordgo.Session, i *discordgo.InteractionCreate) {
			address := i.ApplicationCommandData().Options[0].StringValue()
			log.Info().
				Str("address", address).
				Msg("announcing")

			server, err := GetServerByAddress(i.GuildID, address)
			if err != nil {
				log.Error().
					Str("guild", i.GuildID).
					Str("address", address).
					Err(err).
					Msg("retrieving server failed")
				return
			}
			if server == nil {
				if err := s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
					Type: discordgo.InteractionResponseChannelMessageWithSource,
					Data: &discordgo.InteractionResponseData{
						Content: "No server not found under the specified address and under current guild",
						Flags:   discordgo.MessageFlagsEphemeral,
					},
				}); err != nil {
					panic(err)
				}
			}

			if err := s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Components: []discordgo.MessageComponent{
						discordgo.ActionsRow{
							Components: []discordgo.MessageComponent{
								discordgo.Button{
									Label:    "Register",
									Style:    discordgo.PrimaryButton,
									CustomID: fmt.Sprintf("register/%s", server.ID),
								},
								discordgo.Button{
									Label:    "Update",
									Style:    discordgo.PrimaryButton,
									CustomID: fmt.Sprintf("update/%s", server.ID),
								},
							},
						},
					},
					Content: fmt.Sprintf("View actions below for your Discord account within %s", server.Address),
				},
			}); err != nil {
				panic(err)
			}
		},
	}
)

func authorize(server *Server, angel *Angel, srcAddr string) (bool, error) {
	fmt.Println("auth with angel", angel)
	fmt.Printf("user id: %s\n", angel.UserID)

	channel, err := session.UserChannelCreate(angel.UserID)
	fmt.Printf("err: %v", err)
	if err != nil {
		return false, err
	}
	fmt.Println("got user channel")

	authorizationID := uuid.New()
	fmt.Printf("authorization ID: %s\n", authorizationID.String())
	authorizations[authorizationID.String()] = make(chan bool, 100)
	fmt.Println("sending message")
	_, err = session.ChannelMessageSendComplex(channel.ID, &discordgo.MessageSend{
		Content: fmt.Sprintf("New login request for %s from %s", server.Address, srcAddr),
		Components: []discordgo.MessageComponent{
			discordgo.ActionsRow{
				Components: []discordgo.MessageComponent{
					discordgo.Button{
						Label: "Allow",
						Style: discordgo.PrimaryButton,
						Emoji: discordgo.ComponentEmoji{
							Name: "✅",
						},
						CustomID: fmt.Sprintf("authorization/%s/allow", authorizationID),
					},
					discordgo.Button{
						Label: "Deny",
						Style: discordgo.PrimaryButton,
						Emoji: discordgo.ComponentEmoji{
							Name: "❌",
						},
						CustomID: fmt.Sprintf("authorization/%s/deny", authorizationID),
					},
				},
			},
		},
	})
	if err != nil {
		return false, err
	}
	fmt.Println("waiting for authorization for")
	result := <-authorizations[authorizationID.String()]
	return result, nil
}
