use serde::Serialize;
use std::{ptr, rc::Rc, marker::PhantomData};
use thiserror::Error;
use windows::{
    core::{GUID, Interface, Result as WinResult, HRESULT},
    Win32::{
        Foundation::S_OK,
        Media::Audio::{
            eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator, IMMDevice,
        },
        Media::Audio::Endpoints::IAudioEndpointVolume,
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
        },
    },
};

/// Общие ошибки аудио-модуля.
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("COM initialization failed: {0:?}")]
    ComInit(#[from] windows::core::Error),

    #[error("Windows API error: {0}")]
    WinApi(String),
}

/// Безопасная RAII-обёртка для инициализации COM.
/// 
/// COM инициализация — "на поток". Этот guard:
/// - вызывает `CoInitializeEx` при создании,
/// - вызывает `CoUninitialize` при drop,
/// - не может быть передан между потоками (чтобы избежать ошибок).
pub struct ComGuard {
    _marker: PhantomData<Rc<()>>,
}

impl ComGuard {
    pub fn init() -> Result<Self, AudioError> {
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if hr.is_ok() || hr == S_OK.into() {
            Ok(Self { _marker: PhantomData })
        } else {
            Err(AudioError::ComInit(hr.into()))
        }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

/// Структура, описывающая текущее состояние звука.
#[derive(Serialize, Debug, Clone)]
pub struct AudioState {
    pub volume_percent: u32,
    pub is_muted: bool,
}

/// Активация интерфейса `IAudioEndpointVolume` через vtable.
pub unsafe fn activate_audio_endpoint_volume(device: &IMMDevice) -> WinResult<IAudioEndpointVolume> {
    let iid = &<IAudioEndpointVolume as Interface>::IID;
    let mut ptr = ptr::null_mut();

    let device_ptr = device.as_raw() as *mut std::ffi::c_void;

    let hr = unsafe {
        type ActivateFn = unsafe extern "system" fn(
            *mut std::ffi::c_void,
            &GUID,
            u32,
            *mut std::ffi::c_void,
            *mut *mut std::ffi::c_void,
        ) -> HRESULT;

        let vtable_ptr = *(device_ptr as *mut *mut *mut std::ffi::c_void);
        // [0] QueryInterface, [1] AddRef, [2] Release, [3] Activate
        let activate_ptr = *vtable_ptr.add(3);

        let activate: ActivateFn = std::mem::transmute(activate_ptr);

        activate(device_ptr, iid, CLSCTX_ALL.0, std::ptr::null_mut(), &mut ptr)
    };

    if hr.is_ok() {
        unsafe { Ok(IAudioEndpointVolume::from_raw(ptr)) }
    } else {
        Err(hr.into())
    }
}

/// Получение интерфейса `IAudioEndpointVolume` для устройства вывода по умолчанию.
fn get_endpoint_volume() -> Result<IAudioEndpointVolume, AudioError> {

    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| AudioError::WinApi(format!("CoCreateInstance: {:?}", e)))?;
        let device: IMMDevice = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| AudioError::WinApi(format!("GetDefaultAudioEndpoint: {:?}", e)))?;
        activate_audio_endpoint_volume(&device)
            .map_err(|e| AudioError::WinApi(format!("Activate: {:?}", e)))
    }
}

/// Получение текущего состояния громкости и mute.
pub fn get_audio_state() -> Result<AudioState, AudioError> {
    let _com = ComGuard::init()?;

    unsafe {
        let vol = get_endpoint_volume()?;
        let scalar = vol
            .GetMasterVolumeLevelScalar()
            .map_err(|e| AudioError::WinApi(format!("GetMasterVolumeLevelScalar: {:?}", e)))?;
        let mute = vol
            .GetMute()
            .map_err(|e| AudioError::WinApi(format!("GetMute: {:?}", e)))?;

        Ok(AudioState {
            volume_percent: (scalar * 100.0).round().clamp(0.0, 100.0) as u32,
            is_muted: mute.as_bool(),
        })
    }
}

/// Установка громкости (0–100%)
pub fn set_volume(volume_percent: u32) -> Result<(), AudioError> {
    let _com = ComGuard::init()?;

    unsafe {
        let vol = get_endpoint_volume()?;
        let scalar = (volume_percent.min(100) as f32) / 100.0;
        vol.SetMasterVolumeLevelScalar(scalar, std::ptr::null())
            .map_err(|e| AudioError::WinApi(format!("SetMasterVolumeLevelScalar: {:?}", e)))?;
    }
    Ok(())
}

/// Включить/выключить звук.
pub fn set_mute(mute: bool) -> Result<(), AudioError> {
    let _com = ComGuard::init()?;
    
    unsafe {
        let vol = get_endpoint_volume()?;
        vol.SetMute(mute, std::ptr::null())
            .map_err(|e| AudioError::WinApi(format!("SetMute: {:?}", e)))?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_audio_state_command() -> Result<AudioState, String> {
    get_audio_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_volume_command(volume_percent: u32) -> Result<(), String> {
    set_volume(volume_percent).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_mute_command(mute: bool) -> Result<(), String> {
    set_mute(mute).map_err(|e| e.to_string())
}