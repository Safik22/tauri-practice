use std::{
    ffi::c_void,
    ptr,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{self, Sender},
        Mutex,
    },
    thread,
};
use windows::{
    core::{GUID, Interface},
    Win32::{
        Foundation::{E_NOINTERFACE, S_OK},
        Media::Audio::{
            eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator, IMMDevice,
            AUDIO_VOLUME_NOTIFICATION_DATA,
        },
        Media::Audio::Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback},
        System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED},
    },
};
use tauri::{async_runtime::spawn, AppHandle, Emitter, State};

use crate::audio::{AudioState, activate_audio_endpoint_volume};

#[repr(C)]
struct ComCallback {
    vtbl: *const ComVTable,
    ref_count: AtomicU32,
    app: AppHandle,
}


#[repr(C)]
struct ComVTable {
    query_interface: extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> windows::core::HRESULT,
    add_ref: extern "system" fn(*mut c_void) -> u32,
    release: extern "system" fn(*mut c_void) -> u32,
    on_notify: extern "system" fn(*mut c_void, *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::HRESULT,
}

unsafe fn this_from_ptr(this: *mut c_void) -> *mut ComCallback {
    this as *mut ComCallback
}

extern "system" fn query_interface(this: *mut c_void, riid: *const GUID, ppv: *mut *mut c_void) -> windows::core::HRESULT {
    unsafe {
        if ppv.is_null() {
            return E_NOINTERFACE.into();
        }
        *ppv = ptr::null_mut();

        let iid = &*riid;
        let iid_iunknown = &windows::core::GUID::from_u128(0x00000000_0000_0000_c000_000000000046u128);
        let iid_callback = &<IAudioEndpointVolumeCallback as Interface>::IID;

        if iid == iid_iunknown || iid == iid_callback {
            *ppv = this;
            add_ref(this);
            return S_OK.into();
        }

        E_NOINTERFACE.into()
    }
}

extern "system" fn add_ref(this: *mut c_void) -> u32 {
    unsafe {
        let obj = this_from_ptr(this);
        let old = (*obj).ref_count.fetch_add(1, Ordering::SeqCst);
        old + 1
    }
}

extern "system" fn release(this: *mut c_void) -> u32 {
    unsafe {
        let obj = this_from_ptr(this);
        let prev = (*obj).ref_count.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 {
            let _boxed = Box::from_raw(obj);
            0
        } else {
            prev - 1
        }
    }
}

extern "system" fn on_notify(this: *mut c_void, notify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::HRESULT {
    unsafe {
        if notify.is_null() {
            return S_OK.into();
        }
        let obj = this_from_ptr(this);
        let data = &*notify;

        let state = AudioState {
            volume_percent: (data.fMasterVolume * 100.0).round() as u32,
            is_muted: data.bMuted.as_bool(),
        };

        let app = (*obj).app.clone();
        spawn(async move {
            let _ = app.emit("volume_changed", state);
        });

        S_OK.into()
    }
}

static COM_VTBL: ComVTable = ComVTable {
    query_interface,
    add_ref,
    release,
    on_notify,
};

/// Глобальное состояние наблюдателя.
#[derive(Default)]
pub struct VolumeObserverState {
    pub stop_tx: Mutex<Option<Sender<()>>>,
}

/// Запуск потока с COM callback.
pub fn start_volume_observer_thread(app: AppHandle) -> anyhow::Result<Sender<()>> {
    let (tx, rx) = mpsc::channel::<()>();

    thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
        let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).unwrap();
        let volume: IAudioEndpointVolume =
            activate_audio_endpoint_volume(&device).expect("Activate IAudioEndpointVolume failed");

        let obj = Box::new(ComCallback {
            vtbl: &COM_VTBL,
            ref_count: AtomicU32::new(1),
            app: app.clone(),
        });

        let raw = Box::into_raw(obj);
        let callback_iface = IAudioEndpointVolumeCallback::from_raw(raw as *mut _);

        volume.RegisterControlChangeNotify(&callback_iface).unwrap();

        while rx.recv().is_err() {
            std::thread::park_timeout(std::time::Duration::from_secs(1));
        }

        volume.UnregisterControlChangeNotify(&callback_iface).ok();
        release(raw as *mut _);
        CoUninitialize();
    });

    Ok(tx)
}

/// Запуск наблюдателя и сохранение Sender в состоянии Tauri
pub fn start_volume_observer_manual(app: AppHandle, state: State<VolumeObserverState>) -> anyhow::Result<()> {
    let tx = start_volume_observer_thread(app)?;
    *state.stop_tx.lock().unwrap() = Some(tx);
    Ok(())
}

/// Остановка наблюдателя
#[tauri::command]
pub fn stop_volume_observer_command(state: State<VolumeObserverState>) -> Result<(), String> {
    if let Some(tx) = state.stop_tx.lock().unwrap().take() {
        let _ = tx.send(());
        Ok(())
    } else {
        Err("Observer not running".into())
    }
}