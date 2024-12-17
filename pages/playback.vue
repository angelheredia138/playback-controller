<template>
  <div class="flex items-center justify-center h-screen bg-gray-900 text-white">
    <div class="bg-gray-800 rounded-lg shadow-lg p-6 max-w-sm w-full">
      <!-- Song Image -->
      <img
        :src="song.image"
        alt="Song Artwork"
        class="w-48 h-48 mx-auto rounded-md mb-4"
      />

      <!-- Song Title and Artist -->
      <h2 class="text-xl font-bold text-center">{{ song.title }}</h2>
      <p class="text-md text-gray-400 text-center mb-4">{{ song.artist }}</p>

      <!-- Playback Controls -->
      <div class="flex justify-center items-center space-x-6 mb-4">
        <button
          @click="previousTrack"
          class="text-teal-400 hover:text-teal-300 text-2xl transition"
        >
          ⏮️
        </button>
        <button
          @click="togglePlayPause"
          class="bg-teal-500 hover:bg-teal-400 text-white px-4 py-2 rounded-full text-xl transition"
        >
          {{ isPlaying ? "⏸️" : "▶️" }}
        </button>
        <button
          @click="nextTrack"
          class="text-teal-400 hover:text-teal-300 text-2xl transition"
        >
          ⏭️
        </button>
      </div>

      <!-- Volume Control -->
      <div>
        <p class="text-sm mb-2 text-center">Volume: {{ volume }}%</p>
        <input
          type="range"
          v-model="volume"
          min="0"
          max="100"
          @input="changeVolume"
          class="w-full cursor-pointer"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core"; // Correct import for invoke from Tauri

// Reactive State
const song = ref({
  title: "Song Title",
  artist: "Artist Name",
  image: "https://via.placeholder.com/300", // Placeholder image
});

const isPlaying = ref(false);
const volume = ref(50);

// Fetch the current song from the backend
const getCurrentSong = async () => {
  try {
    const currentSong = await invoke("get_current_song"); // Call the Tauri backend function
    song.value = currentSong; // Update song data with the response
  } catch (error) {
    console.error("Error fetching song:", error);
  }
};

// Playback Controls
const togglePlayPause = () => {
  isPlaying.value = !isPlaying.value;
};

const previousTrack = () => {
  console.log("Previous track");
};

const nextTrack = () => {
  console.log("Next track");
};

const changeVolume = () => {
  console.log("Volume changed:", volume.value);
};

// Fetch song data when the component is mounted
onMounted(() => {
  getCurrentSong(); // Fetch current song from backend
});
</script>
