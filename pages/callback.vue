<template>
  <div class="flex items-center justify-center h-screen bg-gray-900 text-white">
    <div>
      <h1 class="text-2xl mb-4">Loading...</h1>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

// Reactive State
const song = ref({
  title: "Song Title",
  artist: "Artist Name",
  image: "https://via.placeholder.com/300", // Placeholder image
});

const isPlaying = ref(false);
const volume = ref(50);

const getCurrentSong = async () => {
  try {
    const accessToken = localStorage.getItem("spotify_access_token"); // Get access token from localStorage
    if (!accessToken) {
      throw new Error("Access token not found in localStorage.");
    }

    const currentSong = await invoke("get_current_song", { accessToken }); // Pass access token to the backend
    song.value = currentSong; // Update song data with the response
  } catch (error) {
    console.error("Error fetching song:", error);
  }
};

// Fetch song data when the component is mounted
onMounted(() => {
  getCurrentSong(); // Fetch current song from backend
});
</script>
