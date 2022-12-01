package com.gbaranski.heaven;


import net.dv8tion.jda.api.JDA;
import net.dv8tion.jda.api.JDABuilder;
import net.dv8tion.jda.api.entities.Activity;
import net.dv8tion.jda.api.entities.emoji.Emoji;
import net.dv8tion.jda.api.events.interaction.ModalInteractionEvent;
import net.dv8tion.jda.api.events.interaction.component.ButtonInteractionEvent;
import net.dv8tion.jda.api.events.session.ReadyEvent;
import net.dv8tion.jda.api.hooks.ListenerAdapter;
import net.dv8tion.jda.api.interactions.components.buttons.Button;
import net.dv8tion.jda.api.interactions.components.text.TextInput;
import net.dv8tion.jda.api.interactions.components.text.TextInputStyle;
import net.dv8tion.jda.api.interactions.modals.Modal;

import javax.annotation.Nonnull;
import java.util.*;


public class DiscordBot extends ListenerAdapter {
    private static final String ownerID = "398874695069335571";

    private final Configuration configuration;
    private final Database database;
    private final JDA jda;
    private final Map<String, Boolean> authorized = new HashMap<>();


    DiscordBot(Configuration configuration, Database database) {
        this.configuration = configuration;
        this.database = database;
        this.jda = JDABuilder.createDefault(configuration.getToken()).addEventListeners(this).setActivity(Activity.playing("github.com/gbaranski/heaven")).build();
    }

    void awaitReady() {
        try {
            jda.awaitReady();
        } catch (InterruptedException e) {
            throw new RuntimeException(e);
        }
    }

    Boolean authorize(String discordID, String from) {
        var user = jda.getUserById(discordID);
        if (user == null) {
            return false;
        }
        var privateChannel = user.openPrivateChannel().complete();
        var allowButton = Button.primary("authorization/allow", "Allow").withEmoji(Emoji.fromUnicode("✅"));
        var denyButton = Button.secondary("authorization/deny", "Deny").withEmoji(Emoji.fromUnicode("❌"));
        var components = List.of(allowButton, denyButton);
        privateChannel.sendMessage(String.format("New login request for Minecraft server from %s.", from)).setActionRow(components).complete();
        try {
            while (true) {
                authorized.wait();
                var result = authorized.get(discordID);
                if (result != null) {
                    authorized.remove(discordID);
                    return result;
                }
            }
        } catch (InterruptedException e) {
            throw new RuntimeException(e);
        }

    }

    @Override
    public void onReady(@Nonnull ReadyEvent event) {
        var guild = event.getJDA().getGuildById(configuration.getGuildID());
        assert guild != null;
        var channel = guild.getTextChannelById(configuration.getWhitelistChannelID());
        assert channel != null;
        var latestMessageID = channel.getLatestMessageId();
        var latestMessage = channel.retrieveMessageById(latestMessageID).complete();

        var registerButton = Button.primary("register", "Register");
        var changeNicknameButton = Button.secondary("change-nickname", "Change nickname");
        var components = List.of(registerButton, changeNicknameButton);

        var text = String.format("Hello! Tap the button below to register your Discord account within the Minecraft Server.\n> *Bot created by <@%s>*", ownerID);
        if (latestMessage.getAuthor() == event.getJDA().getSelfUser()) {
            latestMessage.editMessage(text).setActionRow(components);
        } else {
            channel.sendMessage(text).setActionRow(components);
        }

    }

    @Override
    public void onButtonInteraction(@Nonnull ButtonInteractionEvent event) {
        var id = event.getInteraction().getId();
        var user = event.getInteraction().getUser();
        switch (id) {
            case "register" -> {
                var angel = database.getAngelByDiscordID(user.getId());
                if (angel != null) {
                    event.reply(String.format("You are already registered under name %s! If that's not your minecraft nickname, please report to the server administrator", angel.minecraftName)).setEphemeral(true).complete();
                    return;
                }
                var minecraftNameInput = TextInput.create("minecraft-name", "Minecraft nickname", TextInputStyle.SHORT).setRequired(true).setLabel("Enter your minecraft nickname").build();
                var components = List.of(minecraftNameInput);
                var modal = Modal.create("registration", "User registration").addActionRow(components).build();
                event.replyModal(modal).complete();
            }
            case "change-nickname" -> {
                var angel = database.getAngelByDiscordID(user.getId());
                if (angel == null) {
                    event.reply("You are not registered at all!").setEphemeral(true).complete();
                    return;
                }
                var newMinecraftNameInput = TextInput.create("minecraft-name", "New Minecraft nickname", TextInputStyle.SHORT).setRequired(true).setLabel("Enter your new minecraft nickname").build();
                var components = List.of(newMinecraftNameInput);
                var modal = Modal.create("nickname-change", "Nickname change").addActionRow(components).build();
                event.replyModal(modal).complete();
            }
            case "authorization/allow" -> {
                authorized.put(user.getId(), true);
                authorized.notify();
            }
            case "authorization/deny" -> {
                authorized.put(user.getId(), false);
                authorized.notify();
            }
            default -> Main.get().getLogger().warning(String.format("Unknown button interaction with ID: %s", id));
        }
    }

    @Override
    public void onModalInteraction(@Nonnull ModalInteractionEvent event) {
        var id = event.getInteraction().getId();
        var user = event.getInteraction().getUser();
        if (id.equals("registration")) {
            var angel = database.getAngelByDiscordID(user.getId());
            if (angel != null) {
                event.reply(String.format("You are already registered under name %s!", angel.minecraftName)).setEphemeral(true).complete();
                return;
            }
            var minecraftName = Objects.requireNonNull(event.getInteraction().getValue("minecraft-name")).getAsString();
            angel = database.getAngelByMinecraftName(minecraftName);
            if (angel != null) {
                event.reply(String.format("Someone's already registered under %s name", minecraftName)).setEphemeral(true).complete();
                return;
            }

            final Angel newAngel = new Angel(user.getId(), user.getName(), minecraftName);
            try {
                database.addAngel(newAngel);
                event.reply(String.format("Thanks for registering! You should be able to connect to the Minecraft server with nickname of %s", minecraftName)).setEphemeral(true).complete();
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
        } else if (id.equals("nickname-change")) {
            var angel = database.getAngelByDiscordID(user.getId());
            if (angel == null) {
                event.reply("You are not registered at all!").setEphemeral(true).complete();
                return;
            }
            var minecraftName = Objects.requireNonNull(event.getInteraction().getValue("new-minecraft-name")).getAsString().trim();
            angel = database.getAngelByMinecraftName(minecraftName);
            angel.minecraftName = minecraftName;
            database.removeAngelByDiscordID(user.getId());
            try {
                database.addAngel(angel);
                event.reply(String.format("Nickname changed! You should be able to connect to the Minecraft server with nickname of %s", minecraftName)).setEphemeral(true).complete();
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
        } else {
            Main.get().getLogger().warning(String.format("Unknown modal interaction with ID: %s", id));
        }
    }
}
