package com.gbaranski.heaven;

import java.util.ArrayList;
import java.util.List;

public class Database {
    private final List<Angel> angels;

    Database() {
        angels = new ArrayList<>();
    }

    Angel getAngelByDiscordID(String discordID) {
        for (Angel angel : angels) {
            if (angel.discordID.equals(discordID)) {
                return angel;
            }
        }
        return null;
    }

    Angel getAngelByMinecraftName(String minecraftName) {
        for (Angel angel : angels) {
            if (angel.minecraftName.equals(minecraftName)) {
                return angel;
            }
        }
        return null;
    }

    void addAngel(Angel newAngel) throws Exception {
        for (Angel angel : angels) {
            if (angel.discordID.equals(newAngel.discordID)) {
                throw new Exception("there's already an angel with this discord id");
            } else if(angel.minecraftName.equals(newAngel.minecraftName)) {
                throw new Exception("there's already an angel with this minecraft name");
            }
        }
        this.angels.add(newAngel);
    }

    void removeAngelByDiscordID(String discordID) {
        for (Angel angel : angels) {
            if (angel.discordID.equals(discordID)) {
                angels.remove(angel);
                return;
            }
        }
    }
}
