<template>
  <div class="flex items-center justify-center h-screen bg-gray-900 text-white">
    <div class="text-center">
      <h1 class="text-2xl mb-4">Logging in...</h1>
      <p v-if="errorMessage" class="text-red-500 mb-4">{{ errorMessage }}</p>
      <p v-else-if="successMessage" class="text-green-500 mb-4">{{ successMessage }}</p>
      <p v-else>Waiting for authorization code...</p>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const errorMessage = ref("");
const successMessage = ref("");

const handleTokens = async () => {
  try {
    // Extract the code from the URL
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get("code");

    if (code) {
      console.log("Authorization Code:", code);

      // Exchange the code for tokens
      const response = await invoke("exchange_spotify_token", { code });
      console.log("Token Response:", response);

      // Save tokens in localStorage
      localStorage.setItem("spotify_access_token", response.access_token);
      if (response.refresh_token) {
        localStorage.setItem("spotify_refresh_token", response.refresh_token);
      }

      // Display success message
      successMessage.value = "Tokens saved successfully! Redirecting...";
      
      // Redirect to playback page
      setTimeout(() => {
        window.location.href = "/playback";
      }, 1000);
    } else {
      throw new Error("Authorization code not found in URL.");
    }
  } catch (error) {
    console.error("Error handling tokens:", error);
    errorMessage.value = error.message || "An unexpected error occurred.";
  }
};

onMounted(() => {
  handleTokens();
});
</script>
