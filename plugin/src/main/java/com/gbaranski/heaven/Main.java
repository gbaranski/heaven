package com.gbaranski.heaven;

import org.bukkit.ChatColor;
import org.bukkit.configuration.serialization.ConfigurationSerialization;
import org.bukkit.plugin.java.JavaPlugin;

import java.util.Objects;

public final class Main extends JavaPlugin
{
    private static Main instance;
    private Storage storage;
    private Client client;

    public Main() {
        Main.instance = this;
    }

    @Override
    public void onLoad() {
        super.onLoad();
    }

    public void onEnable() {
        this.getConfig().options().copyDefaults(true);
        this.saveDefaultConfig();
        this.storage = new Storage();
        this.client = new Client();
        this.getServer().getPluginManager().registerEvents(new Listeners(), this);
        this.getLogger().info("Plugin enabled");
    }

    public void onDisable() {
//        this.storage.save();
        this.getLogger().info("Plugin disabled");
    }

    public static Main get(){
        return instance;
    }

    public Storage getStorage() {
        return this.storage;
    }

    public Client getClient() {
        return this.client;
    }
}