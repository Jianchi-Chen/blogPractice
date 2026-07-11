//! 统一配置中心：从环境变量读取配置，便于测试与部署解耦。

use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    /// JWT 过期秒数
    pub jwt_ttl: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| anyhow::anyhow!("JWT_SECRET must be set and contain at least 32 bytes"))?;
        let jwt_secret = validate_jwt_secret(jwt_secret)?;

        Ok(Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://app.db".into()),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            jwt_secret,
            jwt_ttl: env::var("JWT_TTL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(7 * 24 * 3600),
        })
    }
}

fn validate_jwt_secret(secret: String) -> anyhow::Result<String> {
    anyhow::ensure!(
        secret.len() >= 32 && secret != "please-change-me",
        "JWT_SECRET must be set and contain at least 32 bytes"
    );
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::validate_jwt_secret;

    #[test]
    fn jwt_secret_rejects_insecure_values() {
        assert!(validate_jwt_secret("please-change-me".into()).is_err());
        assert!(validate_jwt_secret("too-short".into()).is_err());
        assert!(validate_jwt_secret("a-secure-test-secret-with-32-bytes".into()).is_ok());
    }
}
