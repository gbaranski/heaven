package com.gbaranski.heaven;

import org.bukkit.ChatColor;
import org.bukkit.configuration.ConfigurationSection;
import org.bukkit.configuration.MemorySection;
import org.bukkit.configuration.file.FileConfiguration;
import org.jetbrains.annotations.Nullable;

import java.util.HashMap;
import java.util.Map;
import java.util.Set;

public class Storage {
    private final Map<String, Angel> angels;

    public Storage() {
        angels = new HashMap<>();
        this.reload();
    }

    public void reload() {
        Main.get().reloadConfig();
        final FileConfiguration config = Main.get().getConfig();
        final ConfigurationSection angelsSection = config.getConfigurationSection("angels");
        if (angelsSection != null) {
            angels.clear();
            Main.get().getLogger().info(String.format("Found %d angels", angelsSection.getValues(false).size()));
            for(Map.Entry<String, Object> entry: angelsSection.getValues(false).entrySet()) {
                final MemorySection memorySection = (MemorySection)entry.getValue();
                final String firstName = memorySection.getString("first-name");
                final String lastName = memorySection.getString("last-name");
                final Angel angel = new Angel(firstName, lastName);
                angels.put(entry.getKey(), angel);
            }
        }
        Main.get().getLogger().info("Config reloaded");
    }

    public void saveAngels() {
        final FileConfiguration config = Main.get().getConfig();
        config.createSection("angels");
        final ConfigurationSection angelsSection = config.getConfigurationSection("angels");
        assert angelsSection != null;
        for (Map.Entry<String, Angel> entry : angels.entrySet()) {
            final ConfigurationSection angelSection = angelsSection.createSection(entry.getKey());
            angelSection.set("first-name", entry.getValue().firstName);
            angelSection.set("last-name", entry.getValue().lastName);
        }
        Main.get().saveConfig();
    }

    @Nullable
    public Angel getAngel(final String name) {
        return angels.get(name);
    }

    public Set<Map.Entry<String, Angel>> getAngels() {
        return angels.entrySet();
    }

    public void addAngel(final String name, final Angel angel) {
        angels.put(name, angel);
        saveAngels();
    }

    public void removeAngel(final String name) {
        angels.remove(name);
        saveAngels();
    }
}
