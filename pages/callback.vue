<template>
  <div class="flex items-center justify-center h-screen bg-gray-900 text-white">
    <div>
      <h1 class="text-2xl mb-4">Logging in...</h1>
      <p v-if="errorMessage" class="text-red-500">{{ errorMessage }}</p>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core"; // Correct Tauri import

const router = useRouter();
const route = useRoute();
const errorMessage = ref("");

const handleCallback = async () => {
  try {
    const code = route.query.code; // Get 'code' from URL
    if (!code) {
      throw new Error("Authorization code is missing.");
    }

    console.log("Authorization Code:", code);

    // Exchange the code for an access token
    const response = await invoke("exchange_spotify_token", { code });

    console.log("Token Response:", response);

    // Save tokens in localStorage
    localStorage.setItem("spotify_access_token", response.access_token);
    localStorage.setItem("spotify_refresh_token", response.refresh_token);

    // Redirect to the playback page
    router.push("/playback");
  } catch (error) {
    console.error("Error handling Spotify callback:", error);
    errorMessage.value = error.message || "An unexpected error occurred.";
  }
};

onMounted(() => {
  handleCallback();
});
</script>
