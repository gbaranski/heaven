package com.gbaranski.heaven;

import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerLoginEvent;

public class Listeners implements Listener {
    @EventHandler
    public void onJoin(PlayerLoginEvent e){

        Player p = e.getPlayer();
        Angel angel = Main.get().getStorage().getAngel(p.getName());
        if (angel == null) {
            e.disallow(PlayerLoginEvent.Result.KICK_WHITELIST, "User not on whitelist");
        } else {
            final String nickname = angel.combined(p.getName());
            p.setPlayerListName(nickname);
            p.setDisplayName(nickname);
            p.setCustomName(nickname);
            p.setCustomNameVisible(true);
        }
    }
}
