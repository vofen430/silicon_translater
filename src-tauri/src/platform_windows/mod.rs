use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionEvent {
    pub text: String,
    pub source: SelectionSource,
    pub bounds: Option<SelectionBounds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionBounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionSource {
    UiAutomation,
    ClipboardFallback,
    OcrPlaceholder,
}

pub struct SelectionWatcher;

impl SelectionWatcher {
    pub fn spawn(sender: tokio::sync::mpsc::Sender<SelectionEvent>) {
        std::thread::spawn(move || {
            info!("selection watcher started");
            #[cfg(target_os = "windows")]
            {
                if let Err(err) = windows_impl::run_selection_loop(sender) {
                    tracing::error!(?err, "selection loop failed");
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                drop(sender);
            }
        });
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::{SelectionBounds, SelectionEvent, SelectionSource};
    use crate::storage::ClipboardBackup;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;
    use tokio::sync::mpsc::Sender;
    use tracing::info;

    pub fn run_selection_loop(sender: Sender<SelectionEvent>) -> anyhow::Result<()> {
        let (hook_tx, hook_rx) = mpsc::channel::<()>();
        std::thread::spawn(move || {
            let _ = hook_tx.send(());
        });

        loop {
            match hook_rx.recv_timeout(Duration::from_millis(250)) {
                Ok(_) => {
                    if let Some(event) = poll_selection()? {
                        let _ = sender.blocking_send(event);
                    }
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        Ok(())
    }

    fn poll_selection() -> anyhow::Result<Option<SelectionEvent>> {
        if let Some(event) = try_uia_selection()? {
            return Ok(Some(event));
        }

        if let Some(event) = try_clipboard_selection()? {
            return Ok(Some(event));
        }

        Ok(Some(SelectionEvent {
            text: String::new(),
            source: SelectionSource::OcrPlaceholder,
            bounds: None,
        }))
    }

    fn try_uia_selection() -> anyhow::Result<Option<SelectionEvent>> {
        info!("UIA selection lookup placeholder");
        Ok(None)
    }

    fn try_clipboard_selection() -> anyhow::Result<Option<SelectionEvent>> {
        info!("clipboard fallback placeholder");
        let _backup = ClipboardBackup::capture()?;
        Ok(None)
    }

    #[allow(dead_code)]
    fn convert_rect_to_bounds(left: i32, top: i32, right: i32, bottom: i32) -> SelectionBounds {
        SelectionBounds {
            left,
            top,
            right,
            bottom,
        }
    }
}
