<template>
  <div class="app-container">
    <div class="system-style-layout">
      <!-- Volume Slider -->
      <div class="volume-control-wrapper">
        <div class="volume-icon">🔊</div>
        <div class="slider-container">
          <input 
            type="range" 
            min="0" 
            max="100" 
            :value="audioState.volume_percent"
            @input="onVolumeInput"
            @change="onVolumeChange"
            :disabled="isLoading"
            class="system-slider"
          />
        </div>
        <div class="volume-value">{{ audioState.volume_percent }}%</div>
      </div>

      <!-- Mute Control -->
      <div class="mute-control-wrapper">
        <button 
          @click="toggleMute"
          :disabled="isLoading"
          :class="['mute-btn', { muted: audioState.is_muted, loading: isLoading }]"
        >
          <span v-if="!isLoading">
            {{ audioState.is_muted ? 'Unmute' : 'Mute' }}
          </span>
          <span v-else class="loading-text">...</span>
        </button>
        <div 
          :class="['mute-indicator', audioState.is_muted ? 'muted' : 'unmuted']"
        >
          <span class="mute-icon">{{ audioState.is_muted ? '🔇' : '🔊' }}</span>
          {{ audioState.is_muted ? 'Muted' : 'Unmuted' }}
          <span v-if="isLoading" class="loading-dots">...</span>
        </div>
      </div>

      <!-- Status -->
      <div class="status-section">
        <div class="status-item">
          <span class="status-label">Real-time updates:</span>
          <span class="status-value active">Active</span>
        </div>
        <div class="status-item">
          <span class="status-label">Last update:</span>
          <span class="status-value">{{ lastUpdateTime }}</span>
        </div>
      </div>

      <!-- Navigation -->
      <div class="navigation-section">
        <button @click="goToPowerControl" class="nav-btn">
          <span class="nav-icon">⏻</span>
          Power Control
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted, onUnmounted } from 'vue'
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

// Emits for navigation
const emit = defineEmits<{
  navigate: [view: 'audio' | 'power']
}>()

// Reactive state
const audioState = reactive({
  volume_percent: 50,
  is_muted: false
})

const isLoading = ref(false)
const lastUpdateTime = ref('Just now')
let unlisten: UnlistenFn | null = null
let volumeInputTimeout: number | null = null

// Interface for audio state
interface AudioState {
  volume_percent: number;
  is_muted: boolean;
}

// Navigation
function goToPowerControl() {
  emit('navigate', 'power')
}

// Update timestamp
function updateTimestamp() {
  const now = new Date()
  lastUpdateTime.value = now.toLocaleTimeString()
}

// Load initial audio state
async function loadAudioState() {
  try {
    const state = await invoke<AudioState>('get_audio_state_command')
    Object.assign(audioState, state)
    updateTimestamp()
  } catch (error) {
    console.error('Failed to load audio state:', error)
  }
}

// Handle volume input (real-time slider movement)
function onVolumeInput(event: Event) {
  const target = event.target as HTMLInputElement
  const newVolume = parseInt(target.value)
  
  // Update only the display value immediately for smooth UX
  const volumeValue = document.querySelector('.volume-value')
  if (volumeValue) {
    volumeValue.textContent = `${newVolume}%`
  }
}

// Handle volume change (when slider is released)
async function onVolumeChange(event: Event) {
  const target = event.target as HTMLInputElement
  const newVolume = parseInt(target.value)
  
  // Clear any existing timeout
  if (volumeInputTimeout) {
    clearTimeout(volumeInputTimeout)
  }
  
  isLoading.value = true
  try {
    await invoke('set_volume_command', { volumePercent: newVolume })
    // UI will update automatically via observer event
  } catch (error) {
    console.error('Failed to set volume:', error)
    await loadAudioState()
  } finally {
    // Keep loading state for a bit to show feedback
    volumeInputTimeout = setTimeout(() => {
      isLoading.value = false
    }, 500)
  }
}

// Toggle mute
async function toggleMute() {
  isLoading.value = true
  try {
    await invoke('set_mute_command', { mute: !audioState.is_muted })
    // UI will update automatically via observer event
  } catch (error) {
    console.error('Failed to toggle mute:', error)
    await loadAudioState()
  } finally {
    setTimeout(() => {
      isLoading.value = false
    }, 500)
  }
}

// Setup event listener for real-time volume changes
async function setupVolumeListener() {
  try {
    unlisten = await listen<AudioState>('volume_changed', (event) => {
      // This is the ONLY place where we update the state from system events
      Object.assign(audioState, event.payload)
      updateTimestamp()
    })
  } catch (error) {
    console.error('Failed to setup volume listener:', error)
  }
}

// Lifecycle
onMounted(async () => {
  await loadAudioState()
  await setupVolumeListener()
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (volumeInputTimeout) {
    clearTimeout(volumeInputTimeout)
  }
})
</script>

<style scoped>
.app-container {
  min-height: 100vh;
  background: #f8f9fa;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  display: flex;
  align-items: center;
  justify-content: center;
}

.system-style-layout {
  width: 100%;
  max-width: 400px;
  background: #ffffff;
  border-radius: 12px;
  box-shadow: 
    0 2px 8px rgba(0, 0, 0, 0.08),
    0 1px 2px rgba(0, 0, 0, 0.04);
  padding: 32px;
  margin: 20px;
}

/* Volume Control */
.volume-control-wrapper {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 32px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.volume-icon {
  font-size: 20px;
  color: #495057;
  width: 24px;
  text-align: center;
}

.slider-container {
  flex: 1;
}

.system-slider {
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: #dee2e6;
  outline: none;
  -webkit-appearance: none;
  transition: all 0.2s ease;
}

.system-slider:hover {
  background: #ced4da;
}

.system-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #495057;
  cursor: pointer;
  border: 2px solid #ffffff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: all 0.15s ease;
}

.system-slider::-webkit-slider-thumb:hover {
  background: #343a40;
  transform: scale(1.1);
}

.system-slider:disabled {
  opacity: 0.6;
}

.system-slider:disabled::-webkit-slider-thumb {
  background: #adb5bd;
  cursor: not-allowed;
  transform: none;
}

.volume-value {
  font-size: 14px;
  font-weight: 500;
  color: #495057;
  min-width: 40px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

/* Mute Control */
.mute-control-wrapper {
  margin-bottom: 24px;
  text-align: center;
}

.mute-btn {
  padding: 12px 30px;
  font-size: 16px;
  cursor: pointer;
  border: none;
  border-radius: 25px;
  background: linear-gradient(45deg, #4CAF50, #45a049);
  color: white;
  font-weight: 600;
  transition: all 0.3s ease;
  box-shadow: 0 4px 15px rgba(0,0,0,0.2);
  position: relative;
  overflow: hidden;
  margin-bottom: 12px;
}

.mute-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0,0,0,0.3);
}

.mute-btn:active:not(:disabled) {
  transform: translateY(0);
}

.mute-btn.muted {
  background: linear-gradient(45deg, #ff4757, #ff3742);
}

.mute-btn.loading {
  background: linear-gradient(45deg, #666, #555);
  cursor: not-allowed;
}

.mute-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none !important;
}

.loading-text {
  animation: pulse 1.5s infinite;
}

.mute-indicator {
  margin-top: 12px;
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 1rem;
  color: #495057;
}

.mute-icon {
  font-size: 1.2rem;
}

.mute-indicator.muted {
  color: #ff6b6b;
}

.mute-indicator.unmuted {
  color: #51cf66;
}

.loading-dots {
  animation: pulse 1.5s infinite;
  margin-left: 5px;
}

/* Status Section */
.status-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-top: 20px;
  border-top: 1px solid #e9ecef;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-label {
  font-weight: 300;
  opacity: 0.9;
  color: #6c757d;
  font-size: 14px;
}

.status-value {
  font-weight: 600;
  color: #495057;
  font-size: 14px;
}

.status-value.active {
  color: #51cf66;
}

/* Navigation Section */
.navigation-section {
  text-align: center;
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #e9ecef;
}

.nav-btn {
  padding: 12px 24px;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  background: #ffffff;
  color: #495057;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.nav-btn:hover {
  background: #f8f9fa;
  border-color: #ced4da;
  transform: translateY(-1px);
}

.nav-icon {
  font-size: 1.1rem;
}

@keyframes pulse {
  0%, 100% { 
    opacity: 1; 
  }
  50% { 
    opacity: 0.5; 
  }
}

/* Responsive design */
@media (max-width: 480px) {
  .system-style-layout {
    padding: 24px;
    margin: 16px;
  }
  
  .volume-control-wrapper {
    flex-direction: column;
    gap: 12px;
    padding: 12px;
  }
  
  .volume-value {
    order: -1;
    text-align: center;
  }
  
  .mute-btn {
    width: 100%;
    padding: 15px;
  }
  
  .nav-btn {
    width: 100%;
    justify-content: center;
  }
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .app-container {
    background: #121212;
  }
  
  .system-style-layout {
    background: #1e1e1e;
    box-shadow: 
      0 2px 8px rgba(0, 0, 0, 0.3),
      0 1px 2px rgba(0, 0, 0, 0.2);
  }
  
  .volume-control-wrapper {
    background: #2d2d2d;
    border-color: #404040;
  }
  
  .volume-icon {
    color: #e0e0e0;
  }
  
  .system-slider {
    background: #404040;
  }
  
  .system-slider:hover {
    background: #4a4a4a;
  }
  
  .system-slider::-webkit-slider-thumb {
    background: #e0e0e0;
    border-color: #1e1e1e;
  }
  
  .system-slider::-webkit-slider-thumb:hover {
    background: #ffffff;
  }
  
  .volume-value {
    color: #e0e0e0;
  }
  
  .mute-indicator {
    color: #e0e0e0;
  }
  
  .status-section {
    border-top-color: #404040;
  }
  
  .status-label {
    color: #b0b0b0;
  }
  
  .status-value {
    color: #e0e0e0;
  }
  
  .navigation-section {
    border-top-color: #404040;
  }
  
  .nav-btn {
    background: #2d2d2d;
    border-color: #404040;
    color: #e0e0e0;
  }
  
  .nav-btn:hover {
    background: #363636;
    border-color: #4a4a4a;
  }
}
</style>