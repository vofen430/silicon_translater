use crate::api_client::{ApiClient, TranslationRequest, TranslationResponse};
use crate::platform_windows::{SelectionEvent, SelectionWatcher};
use crate::storage::{AppSettings, CredentialStore};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[derive(Clone)]
pub struct TranslatorCore {
    api_client: ApiClient,
    settings: Arc<Mutex<AppSettings>>,
    credential_store: CredentialStore,
}

impl TranslatorCore {
    pub fn new() -> Self {
        Self {
            api_client: ApiClient::new(),
            settings: Arc::new(Mutex::new(AppSettings::default())),
            credential_store: CredentialStore::new(),
        }
    }

    pub async fn translate_text(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, crate::api_client::ApiError> {
        let api_key = self.credential_store.read_api_key().ok();
        self.api_client.translate(request, api_key).await
    }

    pub fn update_settings(&self, settings: AppSettings) {
        let mut current = self.settings.lock().expect("settings lock");
        *current = settings;
    }

    pub fn load_settings(&self) -> AppSettings {
        self.settings.lock().expect("settings lock").clone()
    }

    pub fn credential_store(&self) -> &CredentialStore {
        &self.credential_store
    }

    pub fn start_selection_watch(&self, app: &AppHandle) {
        let (tx, mut rx) = mpsc::channel::<SelectionEvent>(32);
        SelectionWatcher::spawn(tx);

        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                info!("selection event" = ?event, "selection update");
                if let Err(err) = UiBridgeEmitter::emit_selection(&app_handle, event) {
                    warn!(?err, "failed to emit selection event");
                }
            }
        });
    }
}

pub struct AppState {
    core: TranslatorCore,
}

impl AppState {
    pub fn new(core: TranslatorCore) -> Self {
        Self { core }
    }

    pub fn core(&self) -> &TranslatorCore {
        &self.core
    }
}

struct UiBridgeEmitter;

impl UiBridgeEmitter {
    fn emit_selection(app: &AppHandle, event: SelectionEvent) -> Result<(), tauri::Error> {
        app.emit_all("selection-event", event)
    }
}
