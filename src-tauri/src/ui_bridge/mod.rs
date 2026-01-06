use crate::api_client::TranslationRequest;
use crate::core::AppState;
use crate::storage::{AppSettings, CredentialStore};
use tauri::{AppHandle, Manager, State};
use tracing::{info, warn};

pub struct UiBridge;

impl UiBridge {
    pub fn start_background(app: &AppHandle) {
        let state = app.state::<AppState>();
        state.core().start_selection_watch(app);
    }

    pub fn toggle_detection(app: &AppHandle) {
        let _ = app.emit_all("toggle-detection", ());
    }

    pub fn trigger_screenshot(app: &AppHandle) {
        let _ = app.emit_all("trigger-screenshot", ());
    }

    pub fn open_settings(app: &AppHandle) {
        let _ = app.emit_all("open-settings", ());
        if let Some(window) = app.get_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub struct AppCommands;

impl AppCommands {
    #[tauri::command]
    pub async fn translate(
        state: State<'_, AppState>,
        request: TranslationRequest,
    ) -> Result<crate::api_client::TranslationResponse, String> {
        state
            .core()
            .translate_text(request)
            .await
            .map_err(|err| err.to_string())
    }

    #[tauri::command]
    pub fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
        state.core().update_settings(settings);
        Ok(())
    }

    #[tauri::command]
    pub fn load_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
        Ok(state.core().load_settings())
    }

    #[tauri::command]
    pub fn set_api_key(state: State<'_, AppState>, api_key: String) -> Result<(), String> {
        state
            .core()
            .credential_store()
            .write_api_key(&api_key)
            .map_err(|err| err.to_string())
    }

    #[tauri::command]
    pub fn read_api_key(state: State<'_, AppState>) -> Result<Option<String>, String> {
        match state.core().credential_store().read_api_key() {
            Ok(value) => Ok(value),
            Err(err) => {
                warn!(?err, "failed to read api key");
                Ok(None)
            }
        }
    }
}

impl From<CredentialStore> for AppCommands {
    fn from(_value: CredentialStore) -> Self {
        info!("initialized app commands");
        AppCommands
    }
}
