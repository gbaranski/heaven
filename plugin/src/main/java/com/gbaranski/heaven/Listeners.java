package com.gbaranski.heaven;

import org.bukkit.ChatColor;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.player.AsyncPlayerPreLoginEvent;
import org.bukkit.event.player.PlayerLoginEvent;

public class Listeners implements Listener {
    private final Database database;
    private final DiscordBot discordBot;

    public Listeners(Database database, DiscordBot discordBot) {
        this.database = database;
        this.discordBot = discordBot;
    }

    @EventHandler(priority = EventPriority.HIGHEST)
    public void onAsyncPlayerPreLoginEventHighest(AsyncPlayerPreLoginEvent event) {
        if (event.getLoginResult() != AsyncPlayerPreLoginEvent.Result.ALLOWED) {
            return;
        }
        String playerName = event.getName();
        try {
            Main.get().getLogger().info(String.format("Authorizing %s.", playerName));
            var angel = database.getAngelByMinecraftName(playerName);
            if (angel == null) {
                Main.get().getLogger().info(String.format("Kicking out %s because angel did not exist", playerName));
                event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_WHITELIST, "Not registered.");
                return;
            }
            var isAuthorized = discordBot.authorize(angel.discordID, event.getAddress().toString());
            if (isAuthorized) {
                Main.get().getLogger().info(String.format("Accepting %s", playerName));
                event.allow();
            } else {
                Main.get().getLogger().info(String.format("Kicking out %s because of login denied on Discord", playerName));
                event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_WHITELIST, "Login denied on Discord");
            }
        } catch (Exception ex) {
            Main.get().getLogger().info(String.format("Internal error: %s", ex));
            event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_OTHER, "Authorization failed due to an internal error.");
            throw new RuntimeException(ex);
        }
    }

    @EventHandler
    public void onLogin(PlayerLoginEvent e) {
        Player p = e.getPlayer();
        try {
            var angel = database.getAngelByMinecraftName(p.getName());
            if (angel == null) {
                Main.get().getLogger().info(String.format("Angel with name %s not found", p.getName()));
                e.disallow(PlayerLoginEvent.Result.KICK_WHITELIST, "User not registered.");
            } else {
                final String nickname = ChatColor.translateAlternateColorCodes('&', String.format("&e%s&c(&6%s&c)&f", angel.discordName.replace(' ', '_'), p.getName()));
                p.setPlayerListName(nickname);
                p.setDisplayName(nickname);
                p.setCustomName(nickname);
                p.setCustomNameVisible(true);
            }

        } catch (Exception ex) {
            Main.get().getLogger().info(String.format("Internal error: %s", ex));
            e.disallow(PlayerLoginEvent.Result.KICK_OTHER, "Internal error.");
            throw new RuntimeException(ex);
        }
    }
}
