import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Элементы DOM
let volumeSlider: HTMLInputElement;  // слайдер громкости
let volumeValue: HTMLElement;        // текстовое значение %
let volumeText: HTMLElement;         // заголовок "Volume: X%"
let muteBtn: HTMLButtonElement;      // кнопка Mute/Unmute
let muteStatus: HTMLElement;         // статус muted/unmuted

// Интерфейс для состояния аудио
interface AudioState {
    volume: number;
    is_muted: boolean;
}

// Загрузка начального состояния
async function loadAudioState() {
    try {
        const audioState = await invoke<AudioState>('get_audio_state');
        updateUI(audioState);
    } catch (error) {
        console.error('Failed to load audio state:', error);
        // Заглушка для тестирования
        updateUI({ volume: 50, is_muted: false });
    }
}

// Обновление UI на основе состояния
function updateUI(state: AudioState) {
    const volume = state.volume;
    const isMuted = state.is_muted;

    // Обновляем слайдер и текстовые значения
    if (volumeSlider && volumeValue && volumeText) {
        volumeSlider.value = volume.toString();
        volumeValue.textContent = `${volume}%`;
        volumeText.textContent = `Volume: ${volume}%`;
    }

    // Обновляем состояние кнопки Mute и статуса
    if (muteBtn && muteStatus) {
        if (isMuted) {
            muteBtn.textContent = 'Unmute';
            muteStatus.textContent = '🔇 Muted';
            muteStatus.className = 'mute-indicator muted';
        } else {
            muteBtn.textContent = 'Mute';
            muteStatus.textContent = '🔊 Unmuted';
            muteStatus.className = 'mute-indicator unmuted';
        }
    }
}

// Установка громкости
async function setVolume(volume: number) {
    try {
        const newState = await invoke<AudioState>('set_volume', { volume });
        updateUI(newState);
    } catch (error) {
        console.error('Failed to set volume:', error);
    }
}

// Переключение mute/unmute
async function toggleMute() {
    try {
        const currentState = await invoke<AudioState>('get_audio_state');
        const newState = await invoke<AudioState>('set_mute', { mute: !currentState.is_muted });
        updateUI(newState);
    } catch (error) {
        console.error('Failed to toggle mute:', error);
    }
}

// Инициализация при загрузке
window.addEventListener('DOMContentLoaded', async () => {
    // Получаем элементы DOM
    volumeSlider = document.getElementById('volume-slider') as HTMLInputElement;
    volumeValue = document.getElementById('volume-value') as HTMLElement;
    volumeText = document.getElementById('volume-text') as HTMLElement;
    muteBtn = document.getElementById('mute-btn') as HTMLButtonElement;
    muteStatus = document.getElementById('mute-status') as HTMLElement;

    // Загружаем начальное состояние
    await loadAudioState();

    // Настраиваем обработчики событий
    if (volumeSlider && volumeValue) {
        volumeSlider.addEventListener('input', (e) => {
            // Мгновенное обновление текста при движении слайдера
            const volume = parseInt((e.target as HTMLInputElement).value);
            volumeValue.textContent = `${volume}%`;
        });

        volumeSlider.addEventListener('change', (e) => {
            // Установка громкости при отпускании слайдера
            const volume = parseInt((e.target as HTMLInputElement).value);
            setVolume(volume);
        });
    }

    if (muteBtn) {
        muteBtn.addEventListener('click', toggleMute);
    }

    console.log('Audio Controller initialized!');
});