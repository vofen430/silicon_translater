use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub model: String,
    pub source_lang: String,
    pub target_lang: String,
    pub enable_detection: bool,
    pub selection_min_len: usize,
    pub selection_max_len: usize,
    pub debounce_ms: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            model: "Qwen/Qwen2.5-7B-Instruct".to_string(),
            source_lang: "自动".to_string(),
            target_lang: "中文".to_string(),
            enable_detection: true,
            selection_min_len: 1,
            selection_max_len: 5000,
            debounce_ms: 200,
        }
    }
}

#[derive(Clone)]
pub struct CredentialStore;

impl CredentialStore {
    pub fn new() -> Self {
        Self
    }

    pub fn write_api_key(&self, api_key: &str) -> Result<(), CredentialError> {
        #[cfg(target_os = "windows")]
        {
            return windows_impl::write_credential(api_key);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = api_key;
            return Err(CredentialError::Unsupported);
        }
    }

    pub fn read_api_key(&self) -> Result<Option<String>, CredentialError> {
        #[cfg(target_os = "windows")]
        {
            return windows_impl::read_credential();
        }
        #[cfg(not(target_os = "windows"))]
        {
            return Ok(None);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("unsupported platform")]
    Unsupported,
    #[error("windows error: {0}")]
    Windows(String),
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::CredentialError;
    use windows::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
        CRED_TYPE_GENERIC,
    };
    use windows::Win32::System::DataExchange::GetClipboardData;
    use windows::Win32::System::DataExchange::{CloseClipboard, OpenClipboard};
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::UI::WindowsAndMessaging::CF_UNICODETEXT;

    const CRED_TARGET: &str = "silicon_translater_api_key";

    pub fn write_credential(api_key: &str) -> Result<(), CredentialError> {
        let mut credential = CREDENTIALW::default();
        let target: Vec<u16> = CRED_TARGET.encode_utf16().chain(std::iter::once(0)).collect();
        credential.Type = CRED_TYPE_GENERIC;
        credential.TargetName = windows::core::PWSTR::from_raw(target.as_ptr() as *mut _);
        credential.CredentialBlobSize = api_key.len() as u32;
        credential.CredentialBlob = api_key.as_ptr() as *mut u8;
        credential.Persist = CRED_PERSIST_LOCAL_MACHINE;

        unsafe { CredWriteW(&credential, 0) }
            .ok()
            .map_err(|err| CredentialError::Windows(format!("{err:?}")))
    }

    pub fn read_credential() -> Result<Option<String>, CredentialError> {
        unsafe {
            let mut cred_ptr = std::ptr::null_mut();
            let target: Vec<u16> = CRED_TARGET.encode_utf16().chain(std::iter::once(0)).collect();
            let result = CredReadW(
                windows::core::PCWSTR::from_raw(target.as_ptr()),
                CRED_TYPE_GENERIC,
                0,
                &mut cred_ptr,
            );

            if result.is_err() {
                return Ok(None);
            }

            let cred = &*cred_ptr;
            let bytes = std::slice::from_raw_parts(
                cred.CredentialBlob,
                cred.CredentialBlobSize as usize,
            );
            let value = String::from_utf8_lossy(bytes).to_string();
            CredFree(cred_ptr as _);
            Ok(Some(value))
        }
    }

    pub fn delete_credential() -> Result<(), CredentialError> {
        let target: Vec<u16> = CRED_TARGET.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            CredDeleteW(
                windows::core::PCWSTR::from_raw(target.as_ptr()),
                CRED_TYPE_GENERIC,
                0,
            )
            .ok()
            .map_err(|err| CredentialError::Windows(format!("{err:?}")))
        }
    }

    pub struct ClipboardBackup {
        pub text: Option<String>,
    }

    impl ClipboardBackup {
        pub fn capture() -> Result<Self, CredentialError> {
            unsafe {
                OpenClipboard(None)
                    .ok()
                    .map_err(|err| CredentialError::Windows(format!("{err:?}")))?;
                let handle = GetClipboardData(CF_UNICODETEXT);
                let text = if handle.0 != 0 {
                    let locked = GlobalLock(handle);
                    let value = if !locked.is_null() {
                        let wide = windows::core::PWSTR::from_raw(locked.0 as *mut _);
                        let len = windows::core::wcslen(wide.0);
                        let slice = std::slice::from_raw_parts(wide.0, len);
                        Some(String::from_utf16_lossy(slice))
                    } else {
                        None
                    };
                    GlobalUnlock(handle);
                    value
                } else {
                    None
                };
                CloseClipboard();
                Ok(Self { text })
            }
        }

        pub fn restore(&self) -> Result<(), CredentialError> {
            unsafe {
                OpenClipboard(None)
                    .ok()
                    .map_err(|err| CredentialError::Windows(format!("{err:?}")))?;
                if let Some(text) = &self.text {
                    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                    let size = wide.len() * std::mem::size_of::<u16>();
                    let handle = GlobalAlloc(GMEM_MOVEABLE, size);
                    let locked = GlobalLock(handle);
                    std::ptr::copy_nonoverlapping(wide.as_ptr(), locked.0 as *mut u16, wide.len());
                    GlobalUnlock(handle);
                    windows::Win32::System::DataExchange::EmptyClipboard()
                        .ok()
                        .map_err(|err| CredentialError::Windows(format!("{err:?}")))?;
                    windows::Win32::System::DataExchange::SetClipboardData(CF_UNICODETEXT, handle);
                }
                CloseClipboard();
                Ok(())
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::ClipboardBackup;

#[cfg(not(target_os = "windows"))]
pub struct ClipboardBackup;

#[cfg(not(target_os = "windows"))]
impl ClipboardBackup {
    pub fn capture() -> Result<Self, CredentialError> {
        Ok(Self)
    }

    pub fn restore(&self) -> Result<(), CredentialError> {
        Ok(())
    }
}
