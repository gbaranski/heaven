package com.gbaranski.heaven;

import com.google.gson.annotations.SerializedName;

import java.util.UUID;

public class Angel {
    public UUID id;
    @SerializedName(value = "discord-id")
    public String discordID;
    @SerializedName(value = "discord-name")
    public String discordName;
    @SerializedName(value = "minecraft-name")
    public String minecraftName;
    @SerializedName(value = "minecraft-type")
    public String minecraftType;
}
