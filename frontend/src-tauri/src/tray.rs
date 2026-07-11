use log::info;
use serde::Serialize;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;

#[derive(Clone, Serialize)]
struct UpdateStatusPayload {
    status: String,
    progress: Option<u32>,
    message: Option<String>,
}

pub fn load_system_tray(app_handle: &App) -> tauri::Result<()> {
    let check_version =
        MenuItem::with_id(app_handle, "check_version", "检查更新", true, None::<&str>)?;
    let quit = MenuItem::with_id(app_handle, "quit", "退出", true, None::<&str>)?;
    let hide = MenuItem::with_id(app_handle, "hide", "隐藏", true, None::<&str>)?;
    let show = MenuItem::with_id(app_handle, "show", "显示", true, None::<&str>)?;
    let menu = Menu::with_items(app_handle, &[&check_version, &hide, &show, &quit])?;
    TrayIconBuilder::with_id("main")
        // 把图标打进二进制，发布时不用带外部文件
        .icon(Image::from_bytes(include_bytes!("../icons/cjc.png"))?)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu_event(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| handle_tray_event(tray.app_handle(), event))
        .build(app_handle)?;

    Ok(())
}

/// 更新系统托盘图标
///
/// # 参数
/// * `app` - Tauri 应用句柄
/// * `icon_path` - 图标文件的完整路径，支持本地文件系统路径
///
/// # 返回值
/// * `Ok(())` - 成功更新图标
/// * `Err(String)` - 更新失败时返回错误信息
///
/// # 示例
/// ```rust
/// update_system_tray_icon(&app_handle, "path/to/new-icon.png")?;
/// ```
pub fn update_system_tray_icon(app: &AppHandle, icon_path: &str) -> Result<(), String> {
    info!("尝试更新系统托盘图标为 {}", icon_path);

    // 从文件路径加载图标
    let icon = Image::from_path(icon_path).map_err(|e| {
        log::warn!("加载图标文件 {} 失败: {}", icon_path, e);
        format!("无法加载图标文件 {}: {}", icon_path, e)
    })?;

    // 获取托盘图标实例（获取默认托盘，如果有多个托盘可以使用 tray_by_id）
    let tray = app
        .tray_by_id("main")
        .ok_or_else(|| "未找到系统托盘实例".to_string())?;

    // 更新托盘图标
    tray.set_icon(Some(icon))
        .map_err(|e| format!("设置托盘图标失败: {}", e))?;

    info!("系统托盘图标已更新为 {}", icon_path);

    Ok(())
}

fn handle_menu_event(app: &AppHandle, event_id: &str) {
    match event_id {
        "quit" => app.exit(0),
        "show" => show_main_window(app),
        "hide" => hide_main_window(app),
        "check_version" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                emit_status(&app_handle, "checking", None, None);
                if let Err(e) = check_version(&app_handle).await {
                    log::warn!("检查更新时发生错误: {}", e);
                    emit_status(&app_handle, "error", None, Some(e));
                }
            });
        }
        _ => {}
    }
}

/// 双击托盘图标显示主窗口
fn handle_tray_event(app: &AppHandle, event: TrayIconEvent) {
    if let TrayIconEvent::DoubleClick { .. } = event {
        show_main_window(app);
    }
}

/// 显示主窗口并聚焦
fn show_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// 隐藏主窗口
fn hide_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

/// 调用检查更新功能, 发送事件到前端
async fn check_version(app: &AppHandle) -> Result<(), String> {
    log::info!("开始检查更新...");
    let updater = app.updater().map_err(|e| {
        log::warn!("获取更新器失败: {}", e);
        format!("获取更新器失败: {}", e)
    })?;

    // 检查更新
    let update = updater.check().await.map_err(|e| {
        log::warn!("检查更新失败: {}", e);
        format!("检查更新失败: {}", e)
    })?;

    let Some(update) = update else {
        emit_status(app, "none", None, Some("未发现新版本".into()));
        return Ok(());
    };

    // 找到更新
    emit_status(
        app,
        "found",
        None,
        Some(format!("发现新版本 {}，开始下载", update.version)),
    );

    // 下载并安装
    let mut last_progress = 0;
    let mut downloaded_bytes = 0_u64;
    let download_result = update
        .download_and_install(
            |downloaded, content_length| {
                downloaded_bytes = downloaded_bytes.saturating_add(downloaded as u64);
                if let Some(total) = content_length.filter(|total| *total > 0) {
                    let progress = (((downloaded_bytes as f64 / total as f64) * 100.0).round()
                        as u32)
                        .min(100);
                    // 仅在进度变化时发送，避免刷屏
                    if progress != last_progress {
                        last_progress = progress;
                        emit_status(app, "downloading", Some(progress), None);
                    }
                }
            },
            || {
                // 下载完成回调
            },
        )
        .await;

    match download_result {
        Ok(_) => {
            emit_status(
                app,
                "done",
                Some(100),
                Some("更新下载完成，正在安装".into()),
            );
        }
        Err(e) => {
            emit_status(app, "error", None, Some(format!("下载或安装失败: {}", e)));
            return Err(format!("下载或安装失败: {}", e));
        }
    }

    log::info!("更新过程完成");
    Ok(())
}

/// 发送更新状态事件到前端
fn emit_status(app: &AppHandle, status: &str, progress: Option<u32>, message: Option<String>) {
    let payload = UpdateStatusPayload {
        status: status.to_string(),
        progress,
        message,
    };
    if let Err(e) = app.emit("update_status", payload) {
        log::warn!("发送更新事件失败: {}", e);
    }
}
