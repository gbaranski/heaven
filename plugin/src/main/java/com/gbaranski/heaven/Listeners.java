package com.gbaranski.heaven;

import org.bukkit.ChatColor;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.player.AsyncPlayerPreLoginEvent;
import org.bukkit.event.player.PlayerLoginEvent;

import java.io.IOException;
import java.net.URISyntaxException;

public class Listeners implements Listener {
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onAsyncPlayerPreLoginEventHighest(AsyncPlayerPreLoginEvent event){
        if (event.getLoginResult() != AsyncPlayerPreLoginEvent.Result.ALLOWED) {
            return;
        }
        String playerName = event.getName();
        try {
            Main.get().getLogger().info(String.format("Authorizing %s.", playerName));
            final var result = Main.get().getClient().authorize(playerName, event.getAddress());
            final boolean isAuthorized = result.getKey();
            final String message = result.getValue();
            if (isAuthorized) {
                Main.get().getLogger().info(String.format("Accepting %s", playerName));
                event.allow();
            } else {
                Main.get().getLogger().info(String.format("Kicking out %s due to %s", playerName, message));
                event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_WHITELIST, message);
            }
        } catch (IOException | URISyntaxException ex) {
            event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_OTHER, "Authorization service was not available");
            throw new RuntimeException(ex);
        }
    }
    @EventHandler
    public void onLogin(PlayerLoginEvent e){
        Player p = e.getPlayer();
        try {
            Angel angel = Main.get().getClient().fetchAngel(p.getName());
            if (angel == null) {
                Main.get().getLogger().info(String.format("Angel with name %s not found", p.getName()));
                e.disallow(PlayerLoginEvent.Result.KICK_WHITELIST, "User not on whitelist");
            } else {
                final String nickname = ChatColor.translateAlternateColorCodes('&', String.format("&e%s&c(&6%s&c)&f", angel.name.replace(' ', '_'), p.getName()));
                p.setPlayerListName(nickname);
                p.setDisplayName(nickname);
                p.setCustomName(nickname);
                p.setCustomNameVisible(true);
            }

        } catch (IOException | URISyntaxException ex) {
            e.disallow(PlayerLoginEvent.Result.KICK_OTHER, "Authorization service was not available");
            throw new RuntimeException(ex);
        }
    }
}
