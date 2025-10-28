<template>
  <div class="app-container">
    <div class="system-style-layout">
      <!-- Header -->
      <div class="header-section">
        <h1 class="app-title">Управление питанием</h1>
      </div>

      <!-- Power Controls -->
      <div class="power-controls-section">
        <div class="power-list">
          <!-- Sleep/Hibernate Button -->
          <button 
            @click="putToSleep"
            :disabled="powerLoading"
            class="power-item sleep-item"
          >
            <div class="item-content">
              <span class="item-icon">⏾</span>
              <div class="item-text">
                <span class="item-title">Спящий режим</span>
              </div>
            </div>
            <div class="sleep-options" v-if="!powerLoading" @click.stop>
              <label class="option-label">
                <input 
                  type="checkbox" 
                  v-model="useHibernate" 
                  class="option-checkbox"
                >
                <span class="option-text">Гибернация</span>
              </label>
            </div>
          </button>

          <!-- Shutdown Button -->
          <button 
            @click="showShutdownConfirm = true"
            :disabled="powerLoading"
            class="power-item shutdown-item"
          >
            <div class="item-content">
              <span class="item-icon">⏻</span>
              <div class="item-text">
                <span class="item-title">Завершение работы</span>
              </div>
            </div>
          </button>

          <!-- Restart Button -->
          <button 
            @click="showRestartConfirm = true"
            :disabled="powerLoading"
            class="power-item restart-item"
          >
            <div class="item-content">
              <span class="item-icon">↻</span>
              <div class="item-text">
                <span class="item-title">Перезагрузка</span>
              </div>
            </div>
          </button>
        </div>
      </div>

      <!-- Status Section -->
      <div class="status-section">
        <div class="status-item">
          <span class="status-label">Статус:</span>
          <span class="status-value active">Готов</span>
        </div>
        <div class="status-item">
          <span class="status-label">Режим:</span>
          <span class="status-value">{{ useHibernate ? 'Гибернация' : 'Сон' }}</span>
        </div>
      </div>

      <!-- Shutdown Confirmation Dialog -->
      <div v-if="showShutdownConfirm" class="dialog-overlay">
        <div class="dialog">
          <div class="dialog-icon">⏻</div>
          <h3>Завершение работы</h3>
          <p>Вы уверены, что хотите выключить компьютер?</p>
          <div class="dialog-buttons">
            <button @click="shutdownComputer" class="confirm-btn shutdown-confirm">
              Выключить
            </button>
            <button @click="showShutdownConfirm = false" class="cancel-btn">
              Отмена
            </button>
          </div>
        </div>
      </div>

      <!-- Restart Confirmation Dialog -->
      <div v-if="showRestartConfirm" class="dialog-overlay">
        <div class="dialog">
          <div class="dialog-icon">↻</div>
          <h3>Перезагрузка</h3>
          <p>Вы уверены, что хотите перезагрузить компьютер?</p>
          <div class="dialog-buttons">
            <button @click="restartComputer" class="confirm-btn restart-confirm">
              Перезагрузить
            </button>
            <button @click="showRestartConfirm = false" class="cancel-btn">
              Отмена
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from "@tauri-apps/api/core";

// Reactive state
const powerLoading = ref(false)
const showShutdownConfirm = ref(false)
const showRestartConfirm = ref(false)
const useHibernate = ref(false)

// Power management functions
async function putToSleep() {
  powerLoading.value = true
  try {
    setTimeout(() => {
      window.close()
    }, 1000)
    
    await invoke('sleep_command', { hibernate: useHibernate.value })
    console.log(`Successfully put computer to ${useHibernate.value ? 'hibernate' : 'sleep'}`)
  } catch (error) {
    console.error(`Failed to put computer to ${useHibernate.value ? 'hibernate' : 'sleep'}:`, error)
    alert(`Не удалось перевести компьютер в ${useHibernate.value ? 'гибернацию' : 'спящий режим'}. Проверьте разрешения системы.`)
    powerLoading.value = false
  }
}

async function shutdownComputer() {
  powerLoading.value = true
  try {
    await invoke('shutdown_command')
    console.log('Successfully initiated shutdown')
  } catch (error) {
    console.error('Failed to shutdown computer:', error)
    alert('Не удалось выключить компьютер. Проверьте разрешения системы.')
    powerLoading.value = false
    showShutdownConfirm.value = false
  }
}

async function restartComputer() {
  powerLoading.value = true
  try {
    await invoke('reboot_command')
    console.log('Successfully initiated restart')
  } catch (error) {
    console.error('Failed to restart computer:', error)
    alert('Не удалось перезагрузить компьютер. Проверьте разрешения системы.')
    powerLoading.value = false
    showRestartConfirm.value = false
  }
}
</script>

<style scoped>
.app-container {
  min-height: 100vh;
  background: #000000;
  font-family: 'Segoe UI', system-ui, sans-serif;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.system-style-layout {
  width: 100%;
  max-width: 400px;
  background: transparent;
}

/* Header Section */
.header-section {
  text-align: center;
  margin-bottom: 32px;
}

.app-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: #ffffff;
  margin: 0;
}

/* Power Controls */
.power-controls-section {
  margin-bottom: 24px;
}

.power-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.power-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 12px 16px;
  border: 1px solid #333333;
  border-radius: 8px;
  background: #1a1a1a;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.power-item:hover:not(:disabled) {
  border-color: #555555;
  background: #2a2a2a;
}

.power-item:active:not(:disabled) {
  background: #333333;
}

.power-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.item-content {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.item-icon {
  font-size: 1.5rem;
  width: 24px;
  text-align: center;
  color: #ffffff;
}

.item-text {
  flex: 1;
}

.item-title {
  font-size: 1rem;
  font-weight: 500;
  color: #ffffff;
  line-height: 1;
}

/* Sleep Options */
.sleep-options {
  margin-left: 12px;
  padding: 4px 8px;
  background: #2a2a2a;
  border-radius: 4px;
  border: 1px solid #333333;
}

.option-label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 0.8rem;
  color: #cccccc;
  white-space: nowrap;
}

.option-checkbox {
  width: 14px;
  height: 14px;
  cursor: pointer;
}

.option-text {
  font-weight: 500;
}

/* Status Section */
.status-section {
  padding: 16px;
  background: #1a1a1a;
  border-radius: 8px;
  border: 1px solid #333333;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.status-item:last-child {
  margin-bottom: 0;
}

.status-label {
  font-weight: 500;
  color: #cccccc;
  font-size: 0.9rem;
}

.status-value {
  font-weight: 600;
  color: #ffffff;
  font-size: 0.9rem;
}

.status-value.active {
  color: #10b981;
}

/* Dialog Styles */
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  background: #1a1a1a;
  border-radius: 12px;
  padding: 24px;
  max-width: 320px;
  width: 90%;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
  text-align: center;
  border: 1px solid #333333;
}

.dialog-icon {
  font-size: 2.5rem;
  margin-bottom: 16px;
  color: #ffffff;
}

.dialog h3 {
  margin: 0 0 12px 0;
  color: #ffffff;
  font-size: 1.2rem;
  font-weight: 600;
}

.dialog p {
  margin: 0 0 20px 0;
  color: #cccccc;
  line-height: 1.5;
  font-size: 0.9rem;
}

.dialog-buttons {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.confirm-btn, .cancel-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 80px;
}

.confirm-btn {
  background: #3b82f6;
  color: white;
}

.confirm-btn:hover {
  background: #2563eb;
}

.shutdown-confirm {
  background: #ef4444;
}

.shutdown-confirm:hover {
  background: #dc2626;
}

.restart-confirm {
  background: #10b981;
}

.restart-confirm:hover {
  background: #059669;
}

.cancel-btn {
  background: #333333;
  color: #ffffff;
  border: 1px solid #555555;
}

.cancel-btn:hover {
  background: #444444;
}

/* Responsive design */
@media (max-width: 640px) {
  .app-container {
    padding: 16px;
  }
  
  .power-item {
    padding: 10px 14px;
  }
  
  .item-icon {
    font-size: 1.3rem;
  }
  
  .item-title {
    font-size: 0.95rem;
  }
  
  .dialog-buttons {
    flex-direction: column;
  }
  
  .confirm-btn, .cancel-btn {
    width: 100%;
  }
}
</style>