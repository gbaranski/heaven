package com.gbaranski.heaven;

import org.bukkit.Bukkit;
import org.bukkit.ChatColor;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.jetbrains.annotations.NotNull;

import java.util.Map;
import java.util.Set;

public class HeavenCommand implements CommandExecutor {
    @Override
    public boolean onCommand(@NotNull CommandSender sender, @NotNull Command command, @NotNull String label, @NotNull String[] args) {
        if(args.length == 0){
            sender.sendMessage(ChatColor.translateAlternateColorCodes('&', "&3Heaven plugin, created by gbaranski"));
            return false;
        }
        final String action = args[0].toLowerCase();
        switch (action) {
            case "add" -> {
                if (!sender.hasPermission("heaven.angel.add")) {
                    sender.sendMessage("No permission");
                    return true;
                }
                final String name = args[1];
                final String firstName = args[2];
                final String lastName = args[3];
                final Angel angel = new Angel(firstName, lastName);
                Main.get().getStorage().addAngel(name, angel);
                sender.sendMessage("Added new angel");
                final Player p = Bukkit.getPlayer(name);
                if (p != null) {
                    final String nickname = angel.combined(p.getName());
                    p.setPlayerListName(nickname);
                    p.setDisplayName(nickname);
                    p.setCustomName(nickname);
                    p.setCustomNameVisible(true);
                }
                return true;
            }
            case "remove" -> {
                if (!sender.hasPermission("heaven.angel.remove")) {
                    sender.sendMessage("No permission");
                    return true;
                }
                final String name = args[1];
                Main.get().getStorage().removeAngel(name);
                sender.sendMessage("Removed angel");
                return true;
            }
            case "info" -> {
                if (!sender.hasPermission("heaven.angel.info")) {
                    sender.sendMessage("No permission");
                    return true;
                }
                final String name = args[1];
                final Angel angel = Main.get().getStorage().getAngel(name);
                if (angel == null) {
                    sender.sendMessage("Angel not found");
                } else {
                    sender.sendMessage(String.format("First name: %s", angel.firstName));
                    sender.sendMessage(String.format("Last name: %s", angel.lastName));
                }
                return true;
            }
            case "list" -> {
                if (!sender.hasPermission("heaven.angel.list")) {
                    sender.sendMessage("No permission");
                    return true;
                }
                final Set<Map.Entry<String, Angel>> angels = Main.get().getStorage().getAngels();
                sender.sendMessage("List:");
                for (Map.Entry<String, Angel> entry : angels) {
                    sender.sendMessage(ChatColor.translateAlternateColorCodes('&', String.format("&e%s &6%s &c%s", entry.getKey(), entry.getValue().firstName, entry.getValue().lastName)));
                }
                return true;
            }
            case "reload" -> {
                Main.get().getStorage().reload();
                sender.sendMessage("Reloaded");
                return true;
            }
            default -> {
                sender.sendMessage("Unknown command action");
                return false;
            }
        }
    }
}
