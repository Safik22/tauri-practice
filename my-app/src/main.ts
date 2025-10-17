import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Элементы DOM
let volumeSlider: HTMLInputElement;
let volumeValue: HTMLElement;
let volumeText: HTMLElement;
let volumeFill: HTMLElement;
let muteBtn: HTMLButtonElement;
let muteStatus: HTMLElement;
let refreshBtn: HTMLButtonElement;

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

    if (volumeSlider && volumeValue && volumeText && volumeFill) {
        volumeSlider.value = volume.toString();
        volumeValue.textContent = `${volume}%`;
        volumeText.textContent = `Volume: ${volume}%`;
        volumeFill.style.width = `${volume}%`;
    }

    if (muteBtn && muteStatus) {
        if (isMuted) {
            muteBtn.textContent = 'Unmute';
            muteStatus.textContent = '🔇 Muted';
            muteStatus.className = 'mute-indicator muted';
            if (volumeFill) volumeFill.classList.add('muted');
        } else {
            muteBtn.textContent = 'Mute';
            muteStatus.textContent = '🔊 Unmuted';
            muteStatus.className = 'mute-indicator unmuted';
            if (volumeFill) volumeFill.classList.remove('muted');
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
    volumeFill = document.getElementById('volume-fill') as HTMLElement;
    muteBtn = document.getElementById('mute-btn') as HTMLButtonElement;
    muteStatus = document.getElementById('mute-status') as HTMLElement;
    refreshBtn = document.getElementById('refresh-btn') as HTMLButtonElement;

    // Загружаем начальное состояние
    await loadAudioState();

    // Настраиваем обработчики событий
    if (volumeSlider && volumeValue) {
        volumeSlider.addEventListener('input', (e) => {
            const volume = parseInt((e.target as HTMLInputElement).value);
            volumeValue.textContent = `${volume}%`;
            if (volumeFill) volumeFill.style.width = `${volume}%`;
        });

        volumeSlider.addEventListener('change', (e) => {
            const volume = parseInt((e.target as HTMLInputElement).value);
            setVolume(volume);
        });
    }

    if (muteBtn) {
        muteBtn.addEventListener('click', toggleMute);
    }

    if (refreshBtn) {
        refreshBtn.addEventListener('click', loadAudioState);
    }

    console.log('Audio Controller initialized!');
});