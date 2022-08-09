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
    private URI serverURI;

    public Storage() {
        serverURI = null;
        try {
            this.reload();
        } catch (URISyntaxException e) {
            throw new RuntimeException(e);
        }
    }


    public void reload() throws URISyntaxException {
        Main.get().reloadConfig();
        final FileConfiguration config = Main.get().getConfig();
        serverURI = new URI(Objects.requireNonNull(config.getString("server-url")));
        Main.get().getLogger().info("Config reloaded");
    }

    public URI getServerURI() {
        return this.serverURI;
    }
}
