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
            if (!Main.get().getClient().authorize(playerName, event.getAddress())) {
                event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_WHITELIST, "Login denied from Discord");
            } else {
                event.allow();
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
                e.disallow(PlayerLoginEvent.Result.KICK_WHITELIST, "User not on whitelist");
            } else {

                final String nickname = ChatColor.translateAlternateColorCodes('&', String.format("&e%s&c(&6%s&c)&f", angel.discordName.replace(' ', '_'), p.getName()));
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
