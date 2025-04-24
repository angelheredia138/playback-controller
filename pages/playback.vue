<template>
  <div
    class="relative min-h-screen w-full flex flex-col overflow-hidden bg-black"
  >
    <!-- Animated Background Transition -->
    <div class="absolute inset-0 z-0 overflow-hidden pointer-events-none">
      <transition
        :name="swipeTransitionName"
        @after-enter="afterBgTransition"
        mode="out-in"
      >
        <div
          v-if="showNewBg"
          key="bg-new"
          class="w-full h-full absolute inset-0"
          :style="bgStyle(newBackgroundImage)"
        ></div>
        <div
          v-else
          key="bg-old"
          class="w-full h-full absolute inset-0"
          :style="bgStyle(oldBackgroundImage)"
        ></div>
      </transition>
    </div>
    <div class="absolute inset-0 z-10 bg-black opacity-50"></div>
    <div class="relative z-20 flex flex-col min-h-screen">
      <!-- Top-right container for playlist info -->
      <div class="absolute top-4 right-4 flex flex-col items-end space-y-2">
        <div class="flex space-x-4">
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
        <!-- User Profile Image (moved directly under logout) -->
        <img
          :src="userProfileImage"
          alt="User Profile"
          class="w-12 h-12 rounded-full border-2 border-green-500 shadow mt-2"
        />
      </div>

      <div class="absolute top-8 left-8 text-left flex items-center space-x-4">
        <div>
          <p class="text-sm text-gray-300 font-semibold">
            PLAYING FROM PLAYLIST:
          </p>
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
          <!-- Playlist Image (moved under dropdown) -->
          <img
            :src="playlistImage"
            alt="Playlist"
            class="w-12 h-12 rounded shadow border border-gray-700 mt-2"
          />
        </div>
      </div>
      <!-- Song Info Section -->
      <div class="flex-1 flex flex-col justify-end ml-16 pb-20">
        <div class="flex items-end">
          <img
            :src="song && song.image ? song.image : PLACEHOLDER_IMAGE"
            alt="Song Artwork"
            class="w-32 h-32 rounded-md shadow-lg"
          />
          <div class="ml-8">
            <h2 class="text-xl font-bold text-white mb-2 font-circular">
              {{ song && song.title ? song.title : "Unknown Title" }}
            </h2>
            <p class="text-l text-gray-300 font-circular">
              {{ song && song.artist ? song.artist : "Unknown Artist" }}
            </p>
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
          <button
            @click="toggleMute"
            class="hover:text-white transition-colors"
          >
            <SpeakerXMarkIcon
              v-if="currentVolume === 0"
              class="w-6 h-6 text-gray-300"
            />
            <SpeakerWaveIcon v-else class="w-6 h-6 text-gray-300" />
          </button>
          <input
            type="range"
            min="0"
            max="100"
            step="1"
            v-model.number="currentVolume"
            @input="debouncedUpdateVolume"
            class="volume-slider w-32 h-2 rounded-lg appearance-none"
            :style="{
              background: `linear-gradient(to right, #1DB954 0%, #1DB954 ${currentVolume}%, #9CA3AF ${currentVolume}%, #9CA3AF 100%)`,
            }"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useRouter } from "vue-router";
import {
  ArrowsRightLeftIcon,
  BackwardIcon,
  PlayIcon,
  PauseIcon,
  ForwardIcon,
  ArrowPathIcon,
  SpeakerWaveIcon,
  SpeakerXMarkIcon, // Add this import
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

const PLACEHOLDER_IMAGE = "https://placehold.co/600x600/222/fff?text=No+Image";

const userProfileImage = ref(PLACEHOLDER_IMAGE);
const playlistImage = ref(PLACEHOLDER_IMAGE);

const selectedPlaylist = ref(null); // <-- Move this above the watcher

// Fetch user profile image
async function fetchUserProfileImage() {
  try {
    const access = await getAccessToken();
    const user = await tauriInvoke("get_user_profile", { access });
    userProfileImage.value =
      user.images && user.images.length > 0
        ? user.images[0].url
        : PLACEHOLDER_IMAGE;
  } catch (err) {
    userProfileImage.value = PLACEHOLDER_IMAGE;
  }
}

// Fetch playlist image
async function fetchPlaylistImage(playlistId) {
  try {
    if (!playlistId) {
      playlistImage.value = PLACEHOLDER_IMAGE;
      return;
    }
    const access = await getAccessToken();
    const imageUrl = await tauriInvoke("get_playlist_image", {
      access,
      playlist_id: playlistId,
    });
    playlistImage.value = imageUrl || PLACEHOLDER_IMAGE;
  } catch (err) {
    playlistImage.value = PLACEHOLDER_IMAGE;
  }
}

// Watch for playlist change to update playlist image
watch(selectedPlaylist, (newId) => {
  fetchPlaylistImage(newId);
});

const song = ref({
  title: "Unknown Title",
  artist: "Unknown Artist",
  image: PLACEHOLDER_IMAGE,
  album_image: PLACEHOLDER_IMAGE,
  artist_image: PLACEHOLDER_IMAGE,
});

const backgroundImage = computed(() => {
  return (
    song.value.artist_image ||
    song.value.album_image ||
    song.value.image ||
    PLACEHOLDER_IMAGE
  );
});

const progressMs = ref(0);
const durationMs = ref(0);
const isPlaying = ref(false);
const isShuffleEnabled = ref(false);
const currentVolume = ref(50);
const previousVolume = ref(50);
const availablePlaylists = ref([]);

let tokenExpiryTime = 0;
let tokenRefreshInterval = null;
let songTimeout = null;
let localTimer = null;
let syncInterval = null; // Add this for the 3s sync interval

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
    const response = await tauriInvoke("refresh_spotify_token", {
      refreshToken,
    });

    if (response && response.access_token) {
      localStorage.setItem("spotify_access_token", response.access_token);
      if (response.refresh_token) {
        localStorage.setItem("spotify_refresh_token", response.refresh_token); // Update only if provided
      }
      tokenExpiryTime = Date.now() + response.expires_in * 1000; // Set new expiry time
      console.log("Access token refreshed successfully.");
    } else {
      throw new Error("Failed to refresh access token. Invalid response.");
    }
  } catch (error) {
    console.error("Error refreshing access token:", error);
  }
}

// Helper to handle 429 errors and retry after delay
async function invokeWithRetry(cmd, args = {}, maxRetries = 2) {
  let attempt = 0;
  while (attempt <= maxRetries) {
    try {
      return await tauriInvoke(cmd, args);
    } catch (err) {
      // Check for 429 error (rate limit)
      if (
        err &&
        (err.status === 429 || (err.message && err.message.includes("429")))
      ) {
        // Try to get Retry-After from error (if available)
        let retryAfter = 1; // default 1 second
        if (err.response && err.response.headers) {
          const ra = err.response.headers["retry-after"];
          if (ra) retryAfter = parseInt(ra, 10) || retryAfter;
        }
        console.warn(`Rate limited (429). Retrying after ${retryAfter}s...`);
        await new Promise((res) => setTimeout(res, retryAfter * 1000));
        attempt++;
      } else {
        throw err;
      }
    }
  }
  throw new Error("Max retries reached for Spotify API call.");
}

async function changePlaylist() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      if (selectedPlaylist.value) {
        await invokeWithRetry("change_playlist", {
          access: access,
          id: selectedPlaylist.value,
        });
        setTimeout(async () => {
          await getCurrentSong();
          await fetchPlaybackState();
        }, 100); // Reduced delay
      }
    }
  } catch (err) {
    console.error("Error changing playlist:", err);
  }
}

async function goFullscreen() {
  try {
    if (process.client) {
      await tauriInvoke("toggle_fullscreen");
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
        await invokeWithRetry("pause", { access });
        isPlaying.value = false;
        stopLocalTimer(); // Explicitly stop timer
      } else {
        await invokeWithRetry("play", { access });
        isPlaying.value = true;
        startLocalTimer(); // Start timer only when playing
      }

      // Always fetch current song and playback state after play/pause
      await getCurrentSong();
      await fetchPlaybackState();
    } else {
      console.warn("Not running in client mode. Skipping togglePlayPause.");
    }
  } catch (err) {
    console.error("Error toggling play/pause:", err);
  }
}

async function nextTrack() {
  swipeDirection.value = "left";
  await _nextTrack();
}

async function previousTrack() {
  swipeDirection.value = "right";
  await _previousTrack();
}

// Save original handlers
const _nextTrack = async function () {
  try {
    if (process.client) {
      const access = await getAccessToken();
      await invokeWithRetry("skip_next", { access });
      setTimeout(async () => {
        await getCurrentSong();
        await fetchPlaybackState();
      }, 100);
    }
  } catch (err) {
    console.error("Error skipping to the next track:", err);
  }
};

const _previousTrack = async function () {
  try {
    if (process.client) {
      const access = await getAccessToken();
      await invokeWithRetry("skip_previous", { access });
      setTimeout(async () => {
        await getCurrentSong();
        await fetchPlaybackState();
      }, 100);
    }
  } catch (err) {
    console.error("Error skipping to the previous track:", err);
  }
};

// --- Animation State ---
const oldBackgroundImage = ref(PLACEHOLDER_IMAGE);
const newBackgroundImage = ref(PLACEHOLDER_IMAGE);
const showNewBg = ref(true);
const swipeDirection = ref("none"); // "left", "right", or "none"
const swipeTransitionName = computed(() =>
  swipeDirection.value === "left"
    ? "swipe-left"
    : swipeDirection.value === "right"
    ? "swipe-right"
    : ""
);

// Helper for background style
function bgStyle(img) {
  return {
    // Reduce oversize and negative offsets to avoid glow on top/left
    width: "100vw",
    height: "100vh",
    left: "0",
    top: "0",
    backgroundImage: `url(${img})`,
    backgroundSize: "cover",
    backgroundPosition: "center",
    filter: "blur(24px) brightness(0.6)",
    transition: "none",
    position: "absolute",
  };
}

// --- Song Change Animation Logic ---
function animateBgSwipe(direction) {
  oldBackgroundImage.value = newBackgroundImage.value;
  newBackgroundImage.value = backgroundImage.value;
  swipeDirection.value = direction;
  showNewBg.value = false;
  // Next tick, show new bg (triggers transition)
  setTimeout(() => {
    showNewBg.value = true;
  }, 10);
}
function afterBgTransition() {
  swipeDirection.value = "none";
}

// --- Watch for song change and trigger animation ---
let lastSongId = ref(null);
watch(
  () => song.value,
  (newSong, oldSong) => {
    if (!oldSong || !newSong) return;
    // Detect song change by comparing title+artist (or use a unique id if available)
    const oldId = oldSong.title + oldSong.artist;
    const newId = newSong.title + newSong.artist;
    if (oldId !== newId) {
      // Default to left swipe (auto-next or skip)
      animateBgSwipe("left");
      lastSongId.value = newId;
    }
  }
);

// --- Hook skip/prev to set direction ---

async function toggleShuffle() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      const newShuffleState = await invokeWithRetry("toggle_shuffle", {
        access,
      });
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
      await invokeWithRetry("restart_song", { access });
      // Always fetch current song and playback state after restart
      await getCurrentSong();
      await fetchPlaybackState();
    } else {
      console.warn("Not running in client mode. Skipping restartSong.");
    }
  } catch (err) {
    console.error("Error restarting the song:", err);
  }
}

// Debounce logic for volume changes
let volumeDebounceTimeout = null;
let lastVolumeValue = currentVolume.value;

function debouncedUpdateVolume() {
  if (volumeDebounceTimeout) clearTimeout(volumeDebounceTimeout);
  volumeDebounceTimeout = setTimeout(() => {
    updateVolume();
  }, 300); // 300ms debounce
}

async function updateVolume() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      await invokeWithRetry("set_volume", {
        access: access,
        volume: currentVolume.value,
      });
    } else {
      console.warn("Not running in client mode. Skipping updateVolume.");
    }
  } catch (err) {
    console.error("Error setting volume:", err);
  }
}

async function toggleMute() {
  try {
    if (process.client) {
      if (currentVolume.value === 0) {
        // Unmute: Restore previous volume
        currentVolume.value = previousVolume.value;
      } else {
        // Mute: Store current volume and set to 0
        previousVolume.value = currentVolume.value;
        currentVolume.value = 0;
      }
      // Update Spotify volume
      await updateVolume();
    }
  } catch (err) {
    console.error("Error toggling mute:", err);
  }
}

async function syncWithSpotify() {
  if (!isPlaying.value) return;
  const access = await getAccessToken();
  const currentSong = await invokeWithRetry("fetch_current_song", { access });
  if (currentSong) {
    progressMs.value = currentSong.progress_ms || 0;
  }
}

async function getCurrentSong() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      const currentSong = await invokeWithRetry("fetch_current_song", {
        access,
      });

      if (currentSong) {
        // Update song information
        song.value = {
          title: currentSong.title || "Unknown Title",
          artist: currentSong.artist || "Unknown Artist",
          image: currentSong.image || PLACEHOLDER_IMAGE,
          album_image:
            currentSong.album_image || currentSong.image || PLACEHOLDER_IMAGE,
          artist_image: currentSong.artist_image || PLACEHOLDER_IMAGE,
        };
        // Precise timing update
        progressMs.value = currentSong.progress_ms || 0;
        durationMs.value = currentSong.duration_ms || 0;

        stopLocalTimer();
        if (durationMs.value > 0 && isPlaying.value) {
          startLocalTimer();
        }

        if (songTimeout) clearTimeout(songTimeout);
        const remainingTime = Math.max(durationMs.value - progressMs.value, 0);

        songTimeout = setTimeout(() => {
          getCurrentSong();
          fetchPlaybackState();
        }, remainingTime);
      }
    }
  } catch (err) {
    console.error("Error fetching the current song:", err);
  }
}

// Add timer control functions here, before they're used
function stopLocalTimer() {
  if (localTimer) {
    clearInterval(localTimer);
    localTimer = null;
  }
  if (syncInterval) {
    clearInterval(syncInterval);
    syncInterval = null;
  }
}

function startLocalTimer() {
  stopLocalTimer();

  if (isPlaying.value) {
    let lastUpdate = performance.now();

    localTimer = setInterval(() => {
      const now = performance.now();
      const elapsed = now - lastUpdate;
      lastUpdate = now;

      if (progressMs.value >= durationMs.value) {
        stopLocalTimer();
        progressMs.value = durationMs.value;
      } else {
        progressMs.value += elapsed;
      }
    }, 50); // Local progress only, no API call

    // Only one sync interval!
    syncInterval = setInterval(async () => {
      if (isPlaying.value) {
        await syncWithSpotify();
      }
    }, 3000);
  }
}

function formatTime(ms) {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

// Add fetchPlaybackState here, before onMounted
async function fetchPlaybackState() {
  try {
    if (process.client) {
      const access = await getAccessToken();
      // Fetch playlists
      const playlists = await invokeWithRetry("fetch_playlists", { access });
      if (playlists?.items) {
        availablePlaylists.value = playlists.items.map((playlist) => ({
          id: playlist.id,
          name: playlist.name,
        }));
      } else {
        console.warn("No playlists found.");
      }
      // Fetch playback state
      const playbackState = await invokeWithRetry("get_playback_state", {
        access,
      });
      const contextUri = playbackState?.context?.uri || null;
      if (contextUri && contextUri.startsWith("spotify:playlist:")) {
        const playlistId = contextUri.split(":")[2];
        selectedPlaylist.value = playlistId;
        // Check if current playlist is in availablePlaylists
        const found = availablePlaylists.value.some((p) => p.id === playlistId);
        if (!found) {
          availablePlaylists.value.unshift({
            id: playlistId,
            name: "Unknown Playlist",
          });
        }
      }
      const volume = playbackState?.device?.volume_percent;
      if (typeof volume === "number") {
        currentVolume.value = volume;
      }
      isPlaying.value = playbackState?.is_playing || false;
      isShuffleEnabled.value = playbackState?.shuffle_state || false;
    } else {
      console.warn("Not running in client mode. process.client check failed.");
    }
  } catch (err) {
    console.error("Error fetching playback state or playlists:", err);
  }
}

onMounted(() => {
  console.log("Initializing backend-log listener...");

  listen("backend-log", (event) => {
    console.log("Backend Log:", event.payload);
  }).catch((err) =>
    console.error("Failed to listen for backend-log events:", err)
  );

  // First fetch playback state to get isPlaying status
  fetchPlaybackState().then(() => {
    // Then get current song which will start timer if needed
    getCurrentSong();
  });

  tokenRefreshInterval = setInterval(async () => {
    try {
      console.log("Attempting periodic token refresh...");
      await refreshAccessToken();
    } catch (err) {
      console.error("Error during periodic token refresh:", err);
    }
  }, 3000000); // 50 minutes in milliseconds

  fetchUserProfileImage();
});

onUnmounted(() => {
  if (localTimer) clearInterval(localTimer);
  if (tokenRefreshInterval) clearInterval(tokenRefreshInterval); // Now this works
  if (songTimeout) clearTimeout(songTimeout);
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

/* Remove the hover effect and update volume slider styles */
.volume-slider {
  outline: none;
  transition: background 0.2s ease;
}

/* Style for the slider thumb */
.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: white;
  cursor: pointer;
  border: none;
}

.volume-slider::-moz-range-thumb {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: white;
  cursor: pointer;
  border: none;
}

/* Add padding to the dropdown menu items */
select option {
  background-color: black; /* Match the dropdown menu background */
  color: white; /* Ensure text is visible */
}

html,
body,
#__nuxt,
#app {
  overflow: hidden !important;
  background: #000 !important;
}

/* Swipe left (next/auto-next) */
.swipe-left-enter-active,
.swipe-left-leave-active {
  transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.5s;
  position: absolute;
  width: 100%;
  height: 100%;
}
.swipe-left-enter-from {
  transform: translateX(100%);
  opacity: 0;
}
.swipe-left-enter-to {
  transform: translateX(0%);
  opacity: 1;
}
.swipe-left-leave-from {
  transform: translateX(0%);
  opacity: 1;
}
.swipe-left-leave-to {
  transform: translateX(-100%);
  opacity: 0;
}

/* Swipe right (previous) */
.swipe-right-enter-active,
.swipe-right-leave-active {
  transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.5s;
  position: absolute;
  width: 100%;
  height: 100%;
}
.swipe-right-enter-from {
  transform: translateX(-100%);
  opacity: 0;
}
.swipe-right-enter-to {
  transform: translateX(0%);
  opacity: 1;
}
.swipe-right-leave-from {
  transform: translateX(0%);
  opacity: 1;
}
.swipe-right-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
