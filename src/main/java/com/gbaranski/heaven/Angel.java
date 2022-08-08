package com.gbaranski.heaven;

import org.bukkit.ChatColor;
import org.bukkit.configuration.serialization.ConfigurationSerializable;
import org.jetbrains.annotations.NotNull;

import java.util.HashMap;
import java.util.Map;

public class Angel {
    public String firstName;
    public String lastName;

    public Angel(String _firstName, String _lastName) {
        firstName = _firstName;
        lastName = _lastName;
    }

    public String combined(final String name) {
        return ChatColor.translateAlternateColorCodes('&', String.format("&e%s_%s&c(&6%s&c)&f", firstName, lastName, name));
    }
}
