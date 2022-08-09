package com.gbaranski.heaven;

import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerLoginEvent;

import java.io.IOException;
import java.net.URISyntaxException;

public class Listeners implements Listener {
    @EventHandler
    public void onJoin(PlayerLoginEvent e){

        Player p = e.getPlayer();
        try {
            Angel angel = Main.get().getClient().fetchAngel(p.getName());
            if (angel == null) {
                e.disallow(PlayerLoginEvent.Result.KICK_WHITELIST, "User not on whitelist");
            } else {
                if (!Main.get().getClient().authorize(angel.id)) {
                    e.disallow(PlayerLoginEvent.Result.KICK_WHITELIST, "Login denied from Discord");
                    return;
                }
                final String nickname = angel.discordName;
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
