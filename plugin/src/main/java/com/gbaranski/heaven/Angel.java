package com.gbaranski.heaven;

import com.google.gson.annotations.SerializedName;

public class Angel {
    @SerializedName(value = "discord-id")
    public String discordID;
    @SerializedName(value = "discord-name")
    public String discordName;
    @SerializedName(value = "minecraft-name")
    public String minecraftName;

    Angel(String discordID, String discordName, String minecraftName) {
        this.discordID = discordID;
        this.discordName = discordName;
        this.minecraftName = minecraftName;
    }
}
