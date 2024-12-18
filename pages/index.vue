<template>
  <div class="flex items-center justify-center h-screen bg-gray-900 text-white">
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
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const errorMessage = ref("");

const redirectToSpotify = async () => {
  try {
    // Directly invoke the Tauri command without checking __TAURI__
    const authUrl = await invoke("get_spotify_auth_url");
    console.log("Spotify Auth URL:", authUrl);

    // Redirect the user to the Spotify authorization page
    window.location.href = authUrl;
  } catch (error) {
    console.error("Error redirecting to Spotify:", error);
    errorMessage.value = error.message;
  }
};
</script>
