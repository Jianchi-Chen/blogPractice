use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::UpdaterExt;

#[derive(Default)]
pub struct UpdateState {
    running: AtomicBool,
}

#[derive(Clone, Serialize)]
struct UpdateStatusPayload {
    status: String,
    progress: Option<u32>,
    message: Option<String>,
}

#[derive(Default, Debug)]
struct DownloadProgress {
    downloaded: u64,
    total: Option<u64>,
    last_emitted: u32,
}

impl DownloadProgress {
    fn add_chunk(&mut self, chunk_size: usize, content_length: Option<u64>) -> Option<u32> {
        self.downloaded = self.downloaded.saturating_add(chunk_size as u64);
        if self.total.is_none() {
            self.total = content_length.filter(|total| *total > 0);
        }
        let total = self.total?;
        let progress = (((self.downloaded as f64 / total as f64) * 100.0).round() as u32).min(100);
        if progress == self.last_emitted {
            return None;
        }
        self.last_emitted = progress;
        Some(progress)
    }
}

#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, UpdateState>,
) -> Result<(), String> {
    state.check(&app).await
}

pub fn spawn_update_check(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<UpdateState>();
        if let Err(error) = state.check(&app).await {
            log::warn!("update failed: {error}");
        }
    });
}

impl UpdateState {
    async fn check(&self, app: &AppHandle) -> Result<(), String> {
        if self.running.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        emit_status(app, "checking", None, None);
        let result = check_and_install(app).await;
        self.running.store(false, Ordering::Release);
        if let Err(error) = &result {
            emit_status(app, "error", None, Some(error.clone()));
        }
        result
    }
}

async fn check_and_install(app: &AppHandle) -> Result<(), String> {
    let updater = app
        .updater()
        .map_err(|error| format!("获取更新器失败: {error}"))?;
    let Some(update) = updater
        .check()
        .await
        .map_err(|error| format!("检查更新失败: {error}"))?
    else {
        emit_status(app, "none", None, Some("未发现新版本".into()));
        return Ok(());
    };

    emit_status(
        app,
        "found",
        None,
        Some(format!("发现新版本 {}，开始下载", update.version)),
    );

    let mut progress = DownloadProgress::default();
    update
        .download_and_install(
            |chunk_size, content_length| {
                if let Some(percent) = progress.add_chunk(chunk_size, content_length) {
                    emit_status(app, "downloading", Some(percent), None);
                }
            },
            || {},
        )
        .await
        .map_err(|error| format!("下载或安装失败: {error}"))?;

    emit_status(
        app,
        "done",
        Some(100),
        Some("更新安装完成，正在重启".into()),
    );
    app.restart();
}

fn emit_status(app: &AppHandle, status: &str, progress: Option<u32>, message: Option<String>) {
    let payload = UpdateStatusPayload {
        status: status.to_string(),
        progress,
        message,
    };
    if let Err(error) = app.emit("update_status", payload) {
        log::warn!("failed to emit update status: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::DownloadProgress;

    #[test]
    fn progress_is_monotonic_and_emitted_only_when_changed() {
        let mut progress = DownloadProgress::default();
        assert_eq!(progress.add_chunk(10, Some(100)), Some(10));
        assert_eq!(progress.add_chunk(0, Some(100)), None);
        assert_eq!(progress.add_chunk(40, Some(100)), Some(50));
        assert_eq!(progress.add_chunk(100, Some(100)), Some(100));
    }

    #[test]
    fn progress_waits_until_content_length_is_known() {
        let mut progress = DownloadProgress::default();
        assert_eq!(progress.add_chunk(10, None), None);
        assert_eq!(progress.add_chunk(10, Some(100)), Some(20));
    }
}
