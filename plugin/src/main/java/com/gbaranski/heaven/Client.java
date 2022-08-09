package com.gbaranski.heaven;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonParser;
import org.jetbrains.annotations.Nullable;

import java.io.IOException;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URI;

public class Client {
    URI getURI() {
        return Main.get().getStorage().getServerURI();
    }

    @Nullable
    public Angel fetchAngel(String minecraftName) throws IOException {
        final URI uri = getURI().resolve("./user/by-minecraft-name" + minecraftName);
        HttpURLConnection con = (HttpURLConnection) uri.toURL().openConnection();
        con.setRequestProperty("Accept", "application/json");
        con.setRequestMethod("GET");
        con.setDoOutput(true);
        con.connect();
        JsonParser jp = new JsonParser();
        JsonElement element = jp.parse(new InputStreamReader(con.getInputStream()));
        Gson gson = new Gson();
        return gson.fromJson(element, Angel.class);
    }
}
