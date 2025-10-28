<template>
  <div id="app">
    <AudioControl v-if="currentView === 'audio'" @navigate="handleNavigation" />
    <PowerControl v-else-if="currentView === 'power'" @navigate="handleNavigation" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import AudioControl from './AudioControl.vue'
import PowerControl from './PowerControl.vue'

const currentView = ref<'audio' | 'power'>('audio')

function handleNavigation(view: 'audio' | 'power') {
  currentView.value = view
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  background: #f8f9fa;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
}
</style>