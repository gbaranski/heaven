package com.gbaranski.heaven;

import org.bukkit.ChatColor;
import org.bukkit.configuration.ConfigurationSection;
import org.bukkit.configuration.MemorySection;
import org.bukkit.configuration.file.FileConfiguration;
import org.jetbrains.annotations.Nullable;

import java.net.MalformedURLException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;
import java.util.HashMap;
import java.util.Map;
import java.util.Objects;
import java.util.Set;

public class Storage {
    private URL serverURL;
    private String serverID;

    public Storage() {
        serverURL = null;
        serverID = null;
        try {
            this.reload();
        } catch (MalformedURLException e) {
            throw new RuntimeException(e);
        }
    }


    public void reload() throws MalformedURLException {
        Main.get().reloadConfig();
        final FileConfiguration config = Main.get().getConfig();
        serverURL = new URL(Objects.requireNonNull(config.getString("server-url")));
        serverID = Objects.requireNonNull(config.getString("server-id"));
        Main.get().getLogger().info("Config reloaded");
    }

    public URL getServerURL() {
        return this.serverURL;
    }

    public String getServerID() {
        return this.serverID;
    }
}
