use serde_json::Value;

#[test]
fn desktop_release_keeps_signed_updater_configuration() {
    let config: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();

    assert_eq!(config["bundle"]["createUpdaterArtifacts"], true);
    assert_eq!(config["plugins"]["updater"]["active"], true);
    let endpoint = config["plugins"]["updater"]["endpoints"][0]
        .as_str()
        .unwrap();
    assert!(endpoint.starts_with("https://github.com/"));
    assert!(!config["plugins"]["updater"]["pubkey"]
        .as_str()
        .unwrap()
        .is_empty());
}

#[test]
fn desktop_bundle_uses_the_shared_frontend() {
    let config: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    assert_eq!(config["build"]["frontendDist"], "../dist");
    assert_eq!(config["build"]["beforeBuildCommand"], "npm run build");
}

#[test]
fn desktop_rust_layer_has_no_business_api_or_database_dependencies() {
    let manifest = include_str!("../Cargo.toml");
    let app_entry = include_str!("../src/lib.rs");

    for dependency in ["sqlx", "jsonwebtoken", "argon2", "reqwest"] {
        assert!(
            !manifest.contains(dependency),
            "unexpected {dependency} dependency"
        );
    }
    for command in ["login", "register", "get_articles", "save_avatar"] {
        assert!(!app_entry.contains(command), "unexpected {command} command");
    }
    assert!(app_entry.contains("updater::check_for_updates"));
}
