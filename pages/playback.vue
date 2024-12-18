<template>
  <!-- Main Container with album background -->
  <div
    class="relative min-h-screen w-full flex flex-col"
    :style="{
      backgroundImage: `linear-gradient(rgba(0,0,0,0.6), rgba(0,0,0,0.6)), url(${song.image})`,
      backgroundSize: 'cover',
      backgroundPosition: 'center',
      backgroundRepeat: 'no-repeat',
    }"
  >
    <!-- Top-right container for playlist info -->
    <div class="absolute top-4 right-4 text-left">
      <p class="text-sm text-gray-300 font-semibold">PLAYING FROM PLAYLIST:</p>
      <div class="mt-2">
        <select
          v-model="selectedPlaylist"
          class="bg-gray-800 text-white p-2 rounded text-sm font-semibold"
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
    </div>

    <!-- Song Info Section -->
    <div class="flex-1 flex flex-col justify-end ml-16 pb-32">
      <div class="flex items-end">
        <img
          :src="song.image"
          alt="Song Artwork"
          class="w-64 h-64 rounded-md shadow-lg"
        />
        <div class="ml-8">
          <h2 class="text-6xl font-bold text-white mb-2 font-circular">
            {{ song.title }}
          </h2>
          <p class="text-3xl text-gray-300 font-circular">{{ song.artist }}</p>
        </div>
      </div>
    </div>
    <!-- Progress Bar -->
    <div class="w-full flex items-center space-x-2 mb-4 px-8">
      <span class="text-sm text-gray-300">{{ formatTime(progressMs) }}</span>
      <div class="relative w-full h-1 bg-gray-500 rounded-lg">
        <!-- The filled portion of the bar (white) -->
        <div
          class="absolute top-0 left-0 h-1 bg-white rounded-lg"
          :style="{ width: (progressMs / durationMs) * 100 + '%' }"
        ></div>
      </div>
      <span class="text-sm text-gray-300">{{ formatTime(durationMs) }}</span>
    </div>

    <!-- Playback Controls and Volume -->
    <div class="w-full px-8 py-16 flex items-center relative">
      <!-- Centered Controls -->
      <div
        class="absolute left-1/2 transform -translate-x-1/2 flex justify-center items-center space-x-6"
      >
        <!-- Shuffle -->
        <button
          :class="{
            'text-green-500': isShuffleEnabled,
            'text-gray-300': !isShuffleEnabled,
          }"
          class="p-2 rounded text-gray-300 hover:scale-105 transition-transform"
          @click="toggleShuffle"
        >
          <ArrowsRightLeftIcon class="w-6 h-6" />
        </button>

        <!-- Previous -->
        <button
          class="p-2 rounded text-gray-300 hover:text-white hover:scale-105 transition-transform"
          @click="previousTrack"
        >
          <BackwardIcon class="w-6 h-6" />
        </button>

        <!-- Play/Pause -->
        <button
          class="p-4 rounded-full bg-white text-black hover:scale-105 transition-transform shadow-lg"
          @click="togglePlayPause"
        >
          <template v-if="isPlaying">
            <PauseIcon class="w-8 h-8" />
          </template>
          <template v-else>
            <PlayIcon class="w-8 h-8" />
          </template>
        </button>

        <!-- Next -->
        <button
          class="p-2 rounded text-gray-300 hover:text-white hover:scale-105 transition-transform"
          @click="nextTrack"
        >
          <ForwardIcon class="w-6 h-6" />
        </button>

        <!-- Restart Song -->
        <button
          class="p-2 rounded text-gray-300 hover:text-white hover:scale-105 transition-transform"
          @click="restartSong"
        >
          <ArrowPathIcon class="w-6 h-6" />
        </button>
      </div>

      <!-- Volume Slider (Right-aligned) -->
      <div class="absolute right-8 flex items-center space-x-2">
        <SpeakerWaveIcon class="w-6 h-6 text-gray-300" />
        <input
          type="range"
          min="0"
          max="100"
          step="1"
          v-model.number="currentVolume"
          @input="updateVolume"
          class="w-32 h-2 bg-gray-300 rounded-lg appearance-none hover:bg-green-500 transition-colors"
          style="outline: none"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

import {
  PlayIcon,
  ForwardIcon,
  BackwardIcon,
  ArrowsRightLeftIcon,
  ArrowPathIcon,
  SpeakerWaveIcon,
  PauseIcon,
} from "@heroicons/vue/24/solid";

const isPlaying = ref(false);
const isShuffleEnabled = ref(false);
const playlists = ref([]);
const selectedPlaylist = ref(null);
const currentVolume = ref(50);
const progressMs = ref(0);
const durationMs = ref(0);

const song = ref({
  title: "Unknown Title",
  artist: "Unknown Artist",
  image: "https://via.placeholder.com/300",
});

let refreshIntervalId = null;

const currentPlaylistName = computed(() => {
  const pl = playlists.value.find((p) => p.id === selectedPlaylist.value);
  return pl ? pl.name : "Unknown Playlist";
});

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
      song.value = {
        title: currentSong.title,
        artist: currentSong.artist,
        image: currentSong.image,
      };
      progressMs.value = currentSong.progress_ms;
      durationMs.value = currentSong.duration_ms;
    } else {
      console.warn("⚠️ No song data returned.");
    }
  } catch (err) {
    console.error("❌ Error fetching the current song:", err);
    // handle token refresh if needed
  }
}

function formatTime(ms) {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
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
      await invoke("change_playlist", { playlistId: selectedPlaylist.value });
    } else {
      console.warn("No playlist selected.");
    }
  } catch (err) {
    console.error("Error changing playlist:", err);
  }
}

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

  // Fetch the playback state to get current volume, isPlaying, shuffle
  try {
    const playbackState = await invoke("get_playback_state");
    const volume = playbackState?.device?.volume_percent;
    if (typeof volume === "number") {
      currentVolume.value = volume;
    }

    isPlaying.value = playbackState.is_playing || false;
    isShuffleEnabled.value = playbackState.shuffle_state || false;
  } catch (err) {
    console.error("Error fetching playback state:", err);
  }

  // Update song info every 1.5 seconds
  refreshIntervalId = setInterval(() => {
    getCurrentSong();
  }, 1000);
});

onBeforeUnmount(() => {
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId);
  }
});
</script>

<style>
@font-face {
  font-family: "Gotham Medium";
  src: url("/assets/Gotham Medium.otf") format("opentype");
}

.font-circular {
  font-family: "Gotham Medium", sans-serif;
}

/* Style for the slider thumb */
input[type="range"]::-webkit-slider-thumb {
  background: #fff;
  border: 2px solid #000;
  border-radius: 50%;
  height: 16px;
  width: 16px;
  cursor: pointer;
}

input[type="range"]::-moz-range-thumb {
  background: #fff;
  border: 2px solid #000;
  border-radius: 50%;
  height: 16px;
  width: 16px;
  cursor: pointer;
}

/* Hover effect for volume slider */
input[type="range"]:hover {
  background: #1db954 !important; /* Spotify green */
}
</style>
