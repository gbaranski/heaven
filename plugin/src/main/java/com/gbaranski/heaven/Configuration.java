package com.gbaranski.heaven;

import org.bukkit.configuration.file.FileConfiguration;

import java.util.Objects;

public class Configuration {
    private String token;
    private String guildID;
    private String whitelistChannelID;

    public Configuration() {
        token = null;
        guildID = null;
        whitelistChannelID = null;
        this.reload();
    }


    public void reload() {
        Main.get().reloadConfig();
        final FileConfiguration config = Main.get().getConfig();
        token = Objects.requireNonNull(config.getString("token"));
        guildID = Objects.requireNonNull(config.getString("guild-id"));
        whitelistChannelID = Objects.requireNonNull(config.getString("whitelist-channel-id"));
        Main.get().getLogger().info("Config reloaded");
    }

    public String getToken() {
        return this.token;
    }
    public String getGuildID() {
        return this.guildID;
    }
    public String getWhitelistChannelID() {
        return this.whitelistChannelID;
    }
}
