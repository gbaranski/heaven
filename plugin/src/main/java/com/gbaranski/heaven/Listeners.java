package com.gbaranski.heaven;

import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerLoginEvent;

import java.io.IOException;

public class Listeners implements Listener {
    @EventHandler
    public void onJoin(PlayerLoginEvent e){

        Player p = e.getPlayer();
        Angel angel;
        try {
            angel = Main.get().getClient().fetchAngel(p.getName());
        } catch (IOException ex) {
            throw new RuntimeException(ex);
        }
        if (angel == null) {
            e.disallow(PlayerLoginEvent.Result.KICK_WHITELIST, "User not on whitelist");
        } else {
            final String nickname = angel.discordName;
            p.setPlayerListName(nickname);
            p.setDisplayName(nickname);
            p.setCustomName(nickname);
            p.setCustomNameVisible(true);
        }
    }
}
