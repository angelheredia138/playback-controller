<template>
  <div class="flex items-center justify-center h-screen bg-gray-900 text-white">
    <div class="bg-gray-800 rounded-lg shadow-lg p-6 max-w-sm w-full">
      <img
        :src="song.image"
        alt="Song Artwork"
        class="w-48 h-48 mx-auto rounded-md mb-4"
      />
      <h2 class="text-xl font-bold text-center truncate">{{ song.title }}</h2>
      <p class="text-md text-gray-400 text-center mb-4 truncate">
        {{ song.artist }}
      </p>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";

const song = ref({
  title: "Unknown Title",
  artist: "Unknown Artist",
  image: "https://via.placeholder.com/300",
});

let refreshIntervalId = null;

async function refreshAccessToken() {
  const refreshToken = localStorage.getItem("spotify_refresh_token");
  if (!refreshToken) {
    console.error("No refresh token available. Redirecting to login...");
    window.location.href = "/";
    return null;
  }

  try {
    const refreshedTokenResponse = await invoke("refresh_spotify_token", {
      refresh_token: refreshToken,
    });
    if (refreshedTokenResponse.refresh_token) {
      localStorage.setItem(
        "spotify_refresh_token",
        refreshedTokenResponse.refresh_token
      );
    }

    await invoke("store_access_token", {
      token: refreshedTokenResponse.access_token,
    });
    localStorage.setItem(
      "spotify_access_token",
      refreshedTokenResponse.access_token
    );
    return refreshedTokenResponse.access_token;
  } catch (error) {
    console.error("Error refreshing token:", error);
    localStorage.clear();
    window.location.href = "/";
    return null;
  }
}

async function initializeAccessToken() {
  const access_token = localStorage.getItem("spotify_access_token");
  if (!access_token) {
    console.error("No access token found. Redirecting to login...");
    window.location.href = "/";
    return;
  }
  await invoke("store_access_token", { token: access_token });
}

async function getCurrentSong() {
  try {
    console.log(
      "🚀 Access Token about to be sent:",
      localStorage.getItem("spotify_access_token")
    );
    const currentSong = await invoke("fetch_current_song");
    console.log("🎶 Fetched Song Data:", currentSong);

    if (currentSong) {
      song.value = currentSong;
    } else {
      console.warn("⚠️ No song data returned.");
    }
  } catch (err) {
    console.error("❌ Error fetching the current song:", err);
    const errorMessage = typeof err === "string" ? err : JSON.stringify(err);

    if (errorMessage.includes("expired")) {
      console.log("🔄 Refreshing access token...");
      const newToken = await refreshAccessToken();
      if (newToken) {
        await getCurrentSong();
      }
    } else {
      console.warn("Unhandled error:", errorMessage);
    }
  }
}

onMounted(async () => {
  await initializeAccessToken();
  await getCurrentSong();

  // Call getCurrentSong every 10 seconds to keep updated
  refreshIntervalId = setInterval(() => {
    getCurrentSong();
  }, 1_500); // 1,500ms = 1.5s
});

onBeforeUnmount(() => {
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId);
  }
});
</script>
