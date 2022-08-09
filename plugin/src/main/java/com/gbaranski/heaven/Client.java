package com.gbaranski.heaven;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonParser;
import org.jetbrains.annotations.Nullable;

import java.io.IOException;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;
import java.util.UUID;

public class Client {
    URL getURL() {
        return Main.get().getStorage().getServerURL();
    }

    @Nullable
    public Angel fetchAngel(String minecraftName) throws IOException, URISyntaxException {
        final URI uri = getURL().toURI().resolve("./angel/by-minecraft-name/" + minecraftName);
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

    @Nullable
    public boolean authorize(UUID angelID) throws IOException, URISyntaxException {
        final URI uri = getURL().toURI().resolve("./authorize/" + angelID);
        HttpURLConnection con = (HttpURLConnection) uri.toURL().openConnection();
        con.setRequestMethod("GET");
        con.connect();
        return con.getResponseCode() == 200;
    }
}
