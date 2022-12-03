package com.gbaranski.heaven;

import com.google.gson.annotations.SerializedName;

import java.util.UUID;

public class Angel {
    @SerializedName(value = "name")
    public String name;
    @SerializedName(value = "user-id")
    public String userID;
    @SerializedName(value = "server-id")
    public String serverID;
    @SerializedName(value = "minecraft-name")
    public String minecraftName;
}
