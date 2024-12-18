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
        @click="togglePlayPause"
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
        :class="{
          'bg-green-500': isShuffleEnabled,
          'bg-gray-700': !isShuffleEnabled,
          'hover:bg-gray-600': true,
        }"
        class="p-2 rounded"
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

      <!-- Playlist Dropdown -->
      <div>
        <select
          v-model="selectedPlaylist"
          class="bg-gray-700 text-white p-2 rounded"
          @change="changePlaylist"
        >
          <option value="" disabled>Select a playlist</option>
          <option
            v-for="playlist in playlists"
            :key="playlist.id"
            :value="playlist.id"
          >
            {{
              playlist.name.length > 30
                ? playlist.name.slice(0, 30) + "..."
                : playlist.name
            }}
          </option>
        </select>
      </div>

      <!-- Volume Slider -->
      <div class="flex items-center space-x-2">
        <SpeakerWaveIcon class="w-6 h-6" />
        <input
          type="range"
          min="0"
          max="100"
          step="1"
          v-model.number="currentVolume"
          @input="updateVolume"
          class="w-32 h-2 bg-gray-700 rounded-lg"
        />

        <span class="text-sm text-gray-300">{{ currentVolume }}%</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";

const isPlaying = ref(false);
const isShuffleEnabled = ref(false);
const playlists = ref([]);
const selectedPlaylist = ref(null);
const currentVolume = ref(50); // default volume at 50 if we don’t have state yet
// Import icons from Heroicons (solid set)
import {
  PlayIcon,
  ForwardIcon,
  BackwardIcon,
  ArrowsRightLeftIcon,
  ArrowPathIcon,
  ListBulletIcon,
  SpeakerWaveIcon,
  PauseIcon,
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
      const newToken = await refreshAccessToken();
      if (newToken) {
        await getCurrentSong();
      }
    } else {
      console.warn("Unhandled error:", errorMessage);
    }
  }
}

async function togglePlayPause() {
  try {
    if (isPlaying.value) {
      await invoke("pause");
      isPlaying.value = false;
    } else {
      await invoke("play");
      isPlaying.value = true;
    }
  } catch (err) {
    console.error("Error toggling play/pause:", err);
  }
}

async function nextTrack() {
  try {
    await invoke("skip_next");
  } catch (err) {
    console.error("Error skipping to the next track:", err);
  }
}

async function previousTrack() {
  try {
    await invoke("skip_previous");
  } catch (err) {
    console.error("Error skipping to the previous track:", err);
  }
}

async function toggleShuffle() {
  try {
    const newShuffleState = await invoke("toggle_shuffle");
    isShuffleEnabled.value = newShuffleState;
  } catch (err) {
    console.error("Error toggling shuffle:", err);
  }
}

async function restartSong() {
  try {
    await invoke("restart_song");
  } catch (err) {
    console.error("Error restarting the song:", err);
  }
}
async function fetchPlaylists() {
  try {
    const userPlaylists = await invoke("fetch_playlists");
    playlists.value = userPlaylists.items.map((playlist) => ({
      id: playlist.id,
      name: playlist.name,
    }));
  } catch (err) {
    console.error("Error fetching playlists:", err);
  }
}
async function fetchCurrentPlayback() {
  try {
    const playbackData = await invoke("get_current_playback");

    const contextUri = playbackData?.context?.uri || null;

    // Extract playlist ID if the context URI is a playlist
    if (contextUri && contextUri.startsWith("spotify:playlist:")) {
      selectedPlaylist.value = contextUri.split(":")[2];
    }
  } catch (err) {
    console.error("Error fetching current playback:", err);
  }
}

async function changePlaylist() {
  try {
    if (selectedPlaylist.value) {
      // Use 'playlistId' to match the backend parameter exactly
      await invoke("change_playlist", { playlistId: selectedPlaylist.value });
    } else {
      console.warn("No playlist selected.");
    }
  } catch (err) {
    console.error("Error changing playlist:", err);
  }
}

async function adjustVolume() {}
async function updateVolume() {
  try {
    await invoke("set_volume", { volume: currentVolume.value });
  } catch (err) {
    console.error("Error setting volume:", err);
  }
}

onMounted(async () => {
  await initializeAccessToken();
  await fetchPlaylists();
  await fetchCurrentPlayback();
  await getCurrentSong();

  // Fetch the playback state to get current volume
  try {
    const playbackState = await invoke("get_playback_state");
    // If a device exists in playbackState, use its volume percent
    const volume = playbackState?.device?.volume_percent;
    if (typeof volume === "number") {
      currentVolume.value = volume;
    }

    isPlaying.value = playbackState.is_playing || false;
    isShuffleEnabled.value = playbackState.shuffle_state || false;
  } catch (err) {
    console.error("Error fetching playback state:", err);
  }

  // Update every 1.5 seconds
  refreshIntervalId = setInterval(() => {
    getCurrentSong();
  }, 1500);
});

onBeforeUnmount(() => {
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId);
  }
});
</script>
