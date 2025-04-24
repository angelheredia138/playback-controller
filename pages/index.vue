<template>
  <div class="flex items-center justify-center h-screen bg-gray-900 text-white">
    <div class="absolute top-4 right-4">
      <!-- Fullscreen Button -->
      <button
  @click="goFullscreen"
  class="absolute top-4 right-4 bg-gray-700 text-white px-4 py-2 rounded hover:bg-gray-600"
>
  Go Fullscreen
</button>

    </div>
    <div class="text-center">
      <h1 class="text-3xl font-bold mb-6">Spotify Playback Controller</h1>
      <button
        @click="redirectToSpotify"
        class="bg-green-500 hover:bg-green-400 text-white font-bold py-2 px-4 rounded"
      >
        Login with Spotify
      </button>
      <p v-if="errorMessage" class="mt-4 text-red-500">{{ errorMessage }}</p>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const errorMessage = ref("");


// Check if we're in a Tauri environment and load appWindow
onMounted(async () => {
  listen("redraw", () => {
    window.location.reload(); // Reloads the frontend to ensure graphical consistency
  });
  console.log("Initializing...");
  listen("backend-log", (event) => {
    console.log("Backend Log:", event.payload);
  }).catch((err) => console.error("Failed to listen for backend-log events:", err));
});

const redirectToSpotify = async () => {
  try {
    const authUrl = await invoke("get_spotify_auth_url");
    console.log("Spotify Auth URL:", authUrl);
    window.location.href = authUrl;
  } catch (error) {
    console.error("Error redirecting to Spotify:", error);
    errorMessage.value = error.message;
  }
};

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

</script>



<style scoped>
button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
