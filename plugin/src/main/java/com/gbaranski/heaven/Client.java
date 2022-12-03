package com.gbaranski.heaven;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonParser;
import org.jetbrains.annotations.Nullable;

import java.io.IOException;
import java.io.InputStreamReader;
import java.net.*;
import java.util.AbstractMap;

public class Client {
    URL getURL() {
        return Main.get().getStorage().getServerURL();
    }

    @Nullable
    public Angel fetchAngel(String minecraftName) throws IOException, URISyntaxException {
        final var path = String.format("/%s/by-minecraft-name/%s", Main.get().getStorage().getServerID(), minecraftName);
        final var baseURL = getURL();
        final var uri = new URI(baseURL.getProtocol(), baseURL.getAuthority(), baseURL.getPath() + path, null, null);
        HttpURLConnection con = (HttpURLConnection) uri.toURL().openConnection();
        con.setRequestProperty("Accept", "application/json");
        con.setRequestMethod("GET");
        con.setDoOutput(true);
        con.connect();
        JsonParser jp = new JsonParser();
        JsonElement element = jp.parse(new InputStreamReader(con.getInputStream()));
        Gson gson = new Gson();
        final var angel = gson.fromJson(element, Angel.class);
        con.disconnect();
        return angel;
    }

    public AbstractMap.SimpleEntry<Boolean, String> authorize(String name, InetAddress address) throws IOException, URISyntaxException {
        URL baseURL = getURL();
        var path = String.format("/%s/by-minecraft-name/%s/authorize", Main.get().getStorage().getServerID(), name);
        String query = "from=" + address.getHostAddress();

        final var uri = new URI(baseURL.getProtocol(), baseURL.getAuthority(), baseURL.getPath() + path, query, null);
        Main.get().getLogger().info(String.format("Authorize URI:%s", uri));
        HttpURLConnection con = (HttpURLConnection) uri.toURL().openConnection();
        con.setRequestMethod("POST");
        con.connect();
        Main.get().getLogger().info(String.format("authorization of %s ended with status = %s", name, con.getResponseCode()));
        final var result = switch (con.getResponseCode()) {
            case 200 -> new AbstractMap.SimpleEntry<>(true, "Login accepted from Discord");
            case 401 -> new AbstractMap.SimpleEntry<>(false, "Login denied from Discord");
            case 404 ->
                    new AbstractMap.SimpleEntry<>(false, "Not registered on Discord. Check if you're logging from a correct nickname.");
            default -> new AbstractMap.SimpleEntry<>(false, String.format("Unknown error: %s", con.getRequestMethod()));
        };
        con.disconnect();
        return result;
    }
}
