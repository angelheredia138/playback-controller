<template>
  <!-- Main Container with album background -->
  <div
    class="relative min-h-screen w-full flex flex-col"
    :style="{
      backgroundImage: `linear-gradient(rgba(0,0,0,0.6), rgba(0,0,0,0.6)), url(${song.artist_image})`,
      backgroundSize: 'cover',
      backgroundPosition: 'center',
      backgroundRepeat: 'no-repeat',
    }"
  >
  <!-- Top-right container for playlist info -->
  <div class="absolute top-4 right-4 flex space-x-4">
  <!-- Fullscreen Button -->
  <button
    @click="goFullscreen"
    class="bg-gray-700 text-white px-4 py-2 rounded hover:bg-gray-600"
  >
    Go Fullscreen
  </button>

  <!-- Logout Button -->
  <button
    @click="logout"
    class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-500"
  >
    Logout
  </button>
</div>

  <div class="absolute top-8 left-8 text-left">
      <p class="text-sm text-gray-300 font-semibold">PLAYING FROM PLAYLIST:</p>
      <div class="mt-2">
        <select
          v-model="selectedPlaylist"
          class="bg-black text-black p-2 rounded text-s font-semibold"
          @change="changePlaylist"
        >
          <option value="" disabled>Select a playlist</option>
          <option
            v-for="playlist in availablePlaylists"
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
    <div class="flex-1 flex flex-col justify-end ml-16 pb-20">
      <div class="flex items-end">
        <img
          :src="song.image"
          alt="Song Artwork"
          class="w-32 h-32 rounded-md shadow-lg"
        />
        <div class="ml-8">
          <h2 class="text-xl font-bold text-white mb-2 font-circular">
            {{ song.title }}
          </h2>
          <p class="text-l text-gray-300 font-circular">{{ song.artist }}</p>
        </div>
      </div>
    </div>
    <!-- Progress Bar -->
    <div class="w-full flex items-center space-x-2 mb-20 px-8">
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
    <div class="w-full px-8 flex items-center relative">
      <!-- Centered Controls -->
      <div
        class="absolute bottom-2 left-1/2 transform -translate-x-1/2 flex justify-center items-center space-x-6"
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
      <div class="absolute right-8 bottom-7 flex items-center space-x-2">
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
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useRouter} from "vue-router";
import {
  ArrowsRightLeftIcon,
  BackwardIcon,
  PlayIcon,
  PauseIcon,
  ForwardIcon,
  ArrowPathIcon,
  SpeakerWaveIcon,
} from "@heroicons/vue/24/solid";

const router = useRouter();

async function logout() {
  try {
    // Clear tokens from localStorage
    localStorage.removeItem("spotify_access_token");
    localStorage.removeItem("spotify_refresh_token");
    console.log("User logged out, tokens cleared.");

    // Navigate to the home page
    router.push("/");
  } catch (err) {
    console.error("Error during logout:", err);
  }
}

const song = ref({
  title: "Unknown Title",
  artist: "Unknown Artist",
  image: "https://via.placeholder.com/300",
});
const progressMs = ref(0);
const durationMs = ref(0);
const isPlaying = ref(false);
const isShuffleEnabled = ref(false);
const currentVolume = ref(50);
const availablePlaylists = ref([]); // Add this line to declare availablePlaylists
const selectedPlaylist = ref(null); // Track the currently selected playlist

let tokenExpiryTime = 0; // Global variable to track when the token will expire

async function getAccessToken() {
  const currentTime = Date.now();

  // Check if the token is expired or about to expire (buffer time of 1 minute)
  if (currentTime >= tokenExpiryTime - 60000) {
    console.log("Access token expired or about to expire. Refreshing...");
    await refreshAccessToken();
  }

  const access = localStorage.getItem("spotify_access_token");
  if (!access) {
    throw new Error("No access token found.");
  }
  return access;
}

async function refreshAccessToken() {
  try {
    const refreshToken = localStorage.getItem("spotify_refresh_token");
    if (!refreshToken) {
      throw new Error("No refresh token found in local storage.");
    }

    // Call your backend or the Spotify API to refresh the token
    const response = await invoke("refresh_spotify_token", { refreshToken });

    if (response && response.access_token) {
  localStorage.setItem("spotify_access_token", response.access_token);
  if (response.refresh_token) {
    localStorage.setItem("spotify_refresh_token", response.refresh_token); // Update only if provided
  }
  tokenExpiryTime = Date.now() + response.expires_in * 1000; // Set new expiry time
  console.log("Access token refreshed successfully.");
}
 else {
      throw new Error("Failed to refresh access token. Invalid response.");
    }
  } catch (error) {
    console.error("Error refreshing access token:", error);
  }
}



// Computed property for current playlist name
const currentPlaylistName = computed(() => {
  const pl = availablePlaylists.value.find((p) => p.id === selectedPlaylist.value);
  return pl ? pl.name : "Unknown Playlist";
});
let tokenRefreshInterval = null;
let fetchInterval = null;

async function fetchPlaybackState() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      // Fetch playlists
      const playlists = await invoke("fetch_playlists", { access });
      if (playlists?.items) {
        availablePlaylists.value = playlists.items.map((playlist) => ({
          id: playlist.id,
          name: playlist.name,
        }));
        console.log("Playlists fetched successfully:", availablePlaylists.value);
      } else {
        console.warn("No playlists found.");
      }
      // Fetch playback state
      const playbackState = await invoke("get_playback_state", { access });
      const contextUri = playbackState?.context?.uri || null;
    if (contextUri && contextUri.startsWith("spotify:playlist:")) {
      selectedPlaylist.value = contextUri.split(":")[2];
    }
      const volume = playbackState?.device?.volume_percent;
      if (typeof volume === "number") {
        currentVolume.value = volume;
      }

      isPlaying.value = playbackState?.is_playing || false;
      isShuffleEnabled.value = playbackState?.shuffle_state || false;

      console.log("Playback state fetched successfully:", playbackState);


    } else {
      console.warn("Not running in client mode. process.client check failed.");
    }
  } catch (err) {
    console.error("Error fetching playback state or playlists:", err);
  }
}


async function changePlaylist() {
  try {
    if (process.client) {
      const access = await getAccessToken();

      if (selectedPlaylist.value) {
        console.log("Changing playlist to ID:", selectedPlaylist.value); // Debug log
        await invoke("change_playlist", {
          access: access, // Pass the access token
          id: selectedPlaylist.value, // Selected playlist ID
        });

        console.log(`Playlist changed to: ${selectedPlaylist.value}`);

        // Fetch the current song and playback state after changing the playlist
        await getCurrentSong(); // Sync UI with the new song
        await fetchPlaybackState(); // Update playback state (e.g., volume, shuffle)
      } else {
        console.warn("No playlist selected.");
      }
    } else {
      console.warn("Not running in client mode. Skipping changePlaylist.");
    }
  } catch (err) {
    console.error("Error changing playlist:", err);
  }
}

async function goFullscreen() {
  try {
    if (process.client) {
      await invoke("toggle_fullscreen");
    } else {
      console.warn("Not running in client mode. Skipping goFullscreen.");
    }
  } catch (err) {
    console.error("Error toggling fullscreen:", err);
  }
}

async function togglePlayPause() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      if (isPlaying.value) {
        await invoke("pause", { access });
        isPlaying.value = false;
        clearInterval(localTimer); // Stop the local timer when paused
      } else {
        await invoke("play", { access });
        isPlaying.value = true;
        startLocalTimer(); // Start the local timer when playing
      }
    } else {
      console.warn("Not running in client mode. Skipping togglePlayPause.");
    }
  } catch (err) {
    console.error("Error toggling play/pause:", err);
  }
}


async function nextTrack() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      await invoke("skip_next", { access });
      await getCurrentSong(); // Fetch new song data
    } else {
      console.warn("Not running in client mode. Skipping nextTrack.");
    }
  } catch (err) {
    console.error("Error skipping to the next track:", err);
  }
}


async function previousTrack() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      await invoke("skip_previous", { access });
      await getCurrentSong(); // Fetch new song data
    } else {
      console.warn("Not running in client mode. Skipping previousTrack.");
    }
  } catch (err) {
    console.error("Error skipping to the previous track:", err);
  }
}


async function toggleShuffle() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      const newShuffleState = await invoke("toggle_shuffle", {access});
      isShuffleEnabled.value = newShuffleState;
    } else {
      console.warn("Not running in client mode. Skipping toggleShuffle.");
    }
  } catch (err) {
    console.error("Error toggling shuffle:", err);
  }
}

async function restartSong() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      await invoke("restart_song", { access });
      progressMs.value = 0; // Reset progress
      startLocalTimer(); // Restart the local timer
    } else {
      console.warn("Not running in client mode. Skipping restartSong.");
    }
  } catch (err) {
    console.error("Error restarting the song:", err);
  }
}

async function updateVolume() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      await invoke("set_volume", {
        access: access, // Access token
        volume: currentVolume.value, // Volume from the slider
      });
    } else {
      console.warn("Not running in client mode. Skipping updateVolume.");
    }
  } catch (err) {
    console.error("Error setting volume:", err);
  }
}

let songTimeout = null; // To manage the dynamic timer

async function getCurrentSong() {
  try {
    if (process.client) {
      const access = await getAccessToken();

      const currentSong = await invoke("fetch_current_song", { access });

      if (currentSong) {
        // Update song information
        song.value = {
          title: currentSong.title || "Unknown Title",
          artist: currentSong.artist || "Unknown Artist",
          image: currentSong.image || "https://via.placeholder.com/300",
          artist_image:
            currentSong.artist_image || "https://via.placeholder.com/300",
        };
        progressMs.value = currentSong.progress_ms || 0;
        durationMs.value = currentSong.duration_ms || 0;

        // Update the progress bar
        startLocalTimer();

        // Clear any existing timeout
        if (songTimeout) clearTimeout(songTimeout);

        // Calculate remaining time for the current song
        const remainingTime = Math.max(
          durationMs.value - progressMs.value,
          0
        );

        console.log(`Next API fetch scheduled in ${remainingTime}ms`);

        // Schedule next API fetch dynamically based on remaining time
        songTimeout = setTimeout(() => {
          getCurrentSong(); // Fetch next song details
          fetchPlaybackState(); // Update playback state
        }, remainingTime);
      } else {
        console.warn("No song data returned from fetch_current_song.");
      }
    } else {
      console.warn("Not running in client mode. process.client check failed.");
    }
  } catch (err) {
    console.error("Error fetching the current song:", err);
  }
}

let localTimer = null;

function startLocalTimer() {
  if (localTimer) clearInterval(localTimer); // Clear any existing timer
  localTimer = setInterval(() => {
    if (progressMs.value >= durationMs.value) {
      clearInterval(localTimer); // Stop the timer when the song ends
      progressMs.value = durationMs.value; // Ensure the progress doesn't exceed duration
    } else {
      progressMs.value += 1000; // Increment by 1 second (1000 ms)
    }
  }, 1000);
}


function formatTime(ms) {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2,"0")}`;
}

onMounted(() => {
  console.log("Initializing backend-log listener...");
  
  listen("backend-log", (event) => {
    console.log("Backend Log:", event.payload);
  }).catch((err) =>
    console.error("Failed to listen for backend-log events:", err)
  );

  // Fetch initial playback state and current song
  fetchPlaybackState();
  getCurrentSong();

  // Periodic token refresh every 50 minutes
  const tokenRefreshInterval = setInterval(async () => {
    try {
      console.log("Attempting periodic token refresh...");
      await refreshAccessToken();
    } catch (err) {
      console.error("Error during periodic token refresh:", err);
    }
  }, 3000000); // 50 minutes in milliseconds

});



onUnmounted(() => {
  if (localTimer) clearInterval(localTimer); // Clear local timer
  if (tokenRefreshInterval) clearInterval(tokenRefreshInterval); // Clear token refresh interval
  if (songTimeout) clearTimeout(songTimeout); // Clear dynamic song timer
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


/* Add padding to the dropdown menu items */
select option {
  background-color: black; /* Match the dropdown menu background */
  color: white; /* Ensure text is visible */
}
</style>
