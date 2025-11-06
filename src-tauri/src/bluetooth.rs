use thiserror::Error;
use windows::{
    Devices::Radios::{Radio, RadioKind, RadioState, RadioAccessStatus},
    Devices::Enumeration::{DeviceInformation, DeviceInformationCollection},
    Devices::Bluetooth::BluetoothDevice,
    core::{Error as WinError, HSTRING},
};
use serde::Serialize;

#[derive(Debug, Error)]
pub enum BluetoothError {
    #[error("Не удалось найти Bluetooth адаптер")]
    NotFound,

    #[error("Windows API error: {0}")]
    WinApi(String),
}

impl From<WinError> for BluetoothError {
    fn from(e: WinError) -> Self {
        BluetoothError::WinApi(format!("{:?}", e))
    }
}

#[derive(Debug, Serialize)]
pub struct BluetoothDeviceInfo {
    pub name: String,
    pub address: String, // MAC
}

/// Включение или выключение Bluetooth через WinRT API
async fn set_bluetooth_state(on: bool) -> Result<(), BluetoothError> {
    let request = Radio::RequestAccessAsync()?.await?;
    if request != RadioAccessStatus::Allowed {
        return Err(BluetoothError::WinApi("Доступ к Radio API запрещён".into()));
    }
    let radios = Radio::GetRadiosAsync()?.await?;
    let bluetooth_radio = radios.into_iter()
        .find(|r| r.Kind().ok() == Some(RadioKind::Bluetooth))
        .ok_or(BluetoothError::NotFound)?;

    let target = if on { RadioState::On } else { RadioState::Off };
    bluetooth_radio.SetStateAsync(target)?.await?;
    Ok(())
}

/// Получение статуса Bluetooth
/// 
/// Функция ищет в списке радио с типом Bluetooth и возвращает его состояние:
/// true, если Bluetooth включён, и false — если выключен или состояние неизвестно.
/// 
async fn get_bluetooth_state() -> Result<bool, BluetoothError> {
    let access = Radio::RequestAccessAsync()?.await?;
    if access != RadioAccessStatus::Allowed {
        return Err(BluetoothError::WinApi("Доступ к Radio API запрещён".into()));
    }
    let radios = Radio::GetRadiosAsync()?.await?;
    for r in radios {
        if r.Kind()? == RadioKind::Bluetooth {
            return Ok(r.State().unwrap_or(RadioState::Off) == RadioState::On);
        }
    }
    Err(BluetoothError::NotFound)
}

/// Получение списка Bluetooth-устройств
/// 
/// Параметр paired определяет, какие устройства будут возвращены:
/// - true — возвращаются только сопряжённые устройства (уже подключённые и известные системе);
/// - false — возвращаются доступные, но несопряжённые устройства.
///
/// Возвращает результат с вектором структур BluetoothDeviceInfo, содержащих имя и MAC-адрес устройства.
///
async fn list_bluetooth_devices(paired: bool) -> Result<Vec<BluetoothDeviceInfo>, BluetoothError> {
    let selector = BluetoothDevice::GetDeviceSelectorFromPairingState(paired)?;
    let collection: DeviceInformationCollection = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.await?;
    let mut devices = Vec::new();

    for i in 0..collection.Size()? {
        let info = collection.GetAt(i)?;
        let name = info.Name()?.to_string_lossy();
        let id_str: String = info.Id()?.to_string_lossy();

        let id_hstring = HSTRING::from(id_str);
        let device = BluetoothDevice::FromIdAsync(&id_hstring)?.await?;
        let address = format!("{:012X}", device.BluetoothAddress()?);

        devices.push(BluetoothDeviceInfo { name, address });
    }
    Ok(devices)
}

#[tauri::command]
pub async fn set_bluetooth_state_command(on: bool) -> Result<(), String> {
    match set_bluetooth_state(on).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_bluetooth_state_command() -> Result<bool, String> {
    match get_bluetooth_state().await {
        Ok(state) => Ok(state),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_bluetooth_devices_command(paired: bool) -> Result<Vec<BluetoothDeviceInfo>, String> {
    match list_bluetooth_devices(paired).await {
        Ok(devices) => Ok(devices),
        Err(e) => Err(e.to_string()),
    }
}