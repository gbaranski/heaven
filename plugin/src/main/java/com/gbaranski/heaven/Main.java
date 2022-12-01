package com.gbaranski.heaven;

import org.bukkit.plugin.java.JavaPlugin;

public final class Main extends JavaPlugin
{
    private static Main instance;

    public Main() {
        Main.instance = this;
    }

    public void onEnable() {
        this.getConfig().options().copyDefaults(true);
        this.saveDefaultConfig();
        Configuration configuration = new Configuration();
        Database database = new Database();
        DiscordBot discordBot = new DiscordBot(configuration, database);
        this.getServer().getPluginManager().registerEvents(new Listeners(database, discordBot), this);
        discordBot.awaitReady();
        this.getLogger().info("Plugin enabled");
    }

    public void onDisable() {
//        this.storage.save();
        this.getLogger().info("Plugin disabled");
    }

    public static Main get() {
        return instance;
    }
}