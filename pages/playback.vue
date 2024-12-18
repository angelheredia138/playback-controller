<template>
  <div
    class="flex flex-col items-center justify-center h-screen bg-gray-900 text-white"
  >
    <div class="bg-gray-800 rounded-lg shadow-lg p-6 max-w-sm w-full mb-4">
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

    <!-- Playback Controls -->
    <div class="flex space-x-4">
      <!-- Previous -->
      <button
        class="bg-gray-700 hover:bg-gray-600 p-2 rounded"
        @click="previousTrack"
      >
        <BackwardIcon class="w-6 h-6" />
      </button>

      <button
        class="bg-gray-700 hover:bg-gray-600 p-2 rounded"
        @click="playPause"
      >
        <template v-if="isPlaying">
          <PauseIcon class="w-6 h-6" />
        </template>
        <template v-else>
          <PlayIcon class="w-6 h-6" />
        </template>
      </button>

      <!-- Next -->
      <button
        class="bg-gray-700 hover:bg-gray-600 p-2 rounded"
        @click="nextTrack"
      >
        <ForwardIcon class="w-6 h-6" />
      </button>

      <!-- Shuffle -->
      <button
        class="bg-gray-700 hover:bg-gray-600 p-2 rounded"
        @click="toggleShuffle"
      >
        <ArrowsRightLeftIcon class="w-6 h-6" />
      </button>

      <!-- Restart Song -->
      <button
        class="bg-gray-700 hover:bg-gray-600 p-2 rounded"
        @click="restartSong"
      >
        <ArrowPathIcon class="w-6 h-6" />
      </button>

      <!-- Change Playlist -->
      <button
        class="bg-gray-700 hover:bg-gray-600 p-2 rounded"
        @click="changePlaylist"
      >
        <ListBulletIcon class="w-6 h-6" />
      </button>

      <!-- Volume Control -->
      <button
        class="bg-gray-700 hover:bg-gray-600 p-2 rounded"
        @click="adjustVolume"
      >
        <SpeakerWaveIcon class="w-6 h-6" />
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";

// Import icons from Heroicons (solid set)
import {
  PlayIcon,
  ForwardIcon,
  BackwardIcon,
  ArrowsRightLeftIcon,
  ArrowPathIcon,
  ListBulletIcon,
  SpeakerWaveIcon,
} from "@heroicons/vue/24/solid";

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
    const currentSong = await invoke("fetch_current_song");

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

// Playback control actions (currently placeholders)
async function playPause() {
  try {
    await invoke("play_pause");
    isPlaying.value = !isPlaying.value; // Toggle the playback state
  } catch (err) {
    console.error("Error toggling play/pause:", err);
  }
}

async function nextTrack() {
  console.log("Next track clicked. Not implemented yet.");
}

async function previousTrack() {
  console.log("Previous track clicked. Not implemented yet.");
}

async function toggleShuffle() {
  console.log("Shuffle clicked. Not implemented yet.");
}

async function restartSong() {
  console.log("Restart song clicked. Not implemented yet.");
}

async function changePlaylist() {
  console.log("Change playlist clicked. Not implemented yet.");
}

async function adjustVolume() {
  console.log("Volume control clicked. Not implemented yet.");
}

onMounted(async () => {
  await initializeAccessToken();
  await getCurrentSong();

  // Update every 1.5 seconds
  refreshIntervalId = setInterval(() => {
    getCurrentSong();
  }, 1_500);
});

onBeforeUnmount(() => {
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId);
  }
});
</script>
