//! 统一配置中心：统一使用应用数据目录存储配置和数据

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri::Manager;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    /// JWT 过期秒数
    pub jwt_ttl: i64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3000,
            jwt_secret: Self::generate_random_secret(),
            jwt_ttl: 7 * 24 * 3600,
        }
    }
}

impl Config {
    /// 加载配置：统一从应用数据目录加载或创建
    pub fn load(app_handle: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        std::fs::create_dir_all(&app_data_dir)?;

        let config_path = app_data_dir.join("config.json");

        if config_path.exists() {
            // 读取现有配置
            let config_str = std::fs::read_to_string(&config_path)?;
            Ok(serde_json::from_str(&config_str)?)
        } else {
            // 创建默认配置
            let config = Self::default();

            // 保存配置
            let config_str = serde_json::to_string_pretty(&config)?;
            std::fs::write(&config_path, config_str)?;

            Ok(config)
        }
    }

    /// 生成随机 JWT secret
    fn generate_random_secret() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// 获取数据库路径（统一使用 app data 目录）
    pub fn get_database_path(&self, app_handle: &AppHandle) -> String {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");

        // 确保目录存在
        if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
            eprintln!("Failed to create app data directory: {}", e);
            log::error!("Failed to create app data directory: {}", e);
        } else {
            log::info!("App data directory: {}", app_data_dir.display());
        }

        let db_path = app_data_dir.join("app.db");

        // 统一使用正斜杠，SQLite 在所有平台都支持
        let path_str = db_path
            .to_str()
            .expect("Invalid database path")
            .replace('\\', "/");

        let db_url = format!("sqlite://{}", path_str);
        log::info!("Generated database URL: {}", db_url);

        db_url
    }
}
