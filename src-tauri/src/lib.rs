use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

mod adapters;
mod models;
use models::{StoreItem, StoreProvider};

struct RuntimeState {
    store_catalog: Mutex<Vec<StoreItem>>,
    library_apps: Mutex<Vec<LibraryApp>>,
    runtime_config: Mutex<RuntimeConfig>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            store_catalog: Mutex::new(build_store_catalog()),
            library_apps: Mutex::new(Vec::new()),
            runtime_config: Mutex::new(RuntimeConfig {
                low_resource_mode: true,
                ingestion_enabled: false,
                sync_interval_sec: 30,
            }),
        }
    }
}

#[derive(Serialize, Clone)]
struct LibraryApp {
    id: String,
    name: String,
    category: String,
    #[serde(rename = "providerName")]
    provider_name: String,
    version: String,
    state: String,
    #[serde(rename = "lastLaunched")]
    last_launched: Option<String>,
}

#[derive(Serialize, Clone)]
struct StoreRefreshResult {
    items: usize,
    providers: usize,
}

#[derive(Serialize, Clone)]
struct RuntimeConfig {
    #[serde(rename = "lowResourceMode")]
    low_resource_mode: bool,
    #[serde(rename = "ingestionEnabled")]
    ingestion_enabled: bool,
    #[serde(rename = "syncIntervalSec")]
    sync_interval_sec: u32,
}

#[derive(Serialize, Clone)]
struct RuntimeOverview {
    #[serde(rename = "lowResourceMode")]
    low_resource_mode: bool,
    #[serde(rename = "ingestionEnabled")]
    ingestion_enabled: bool,
    #[serde(rename = "syncIntervalSec")]
    sync_interval_sec: u32,
    #[serde(rename = "libraryCount")]
    library_count: usize,
    #[serde(rename = "runningCount")]
    running_count: usize,
}

#[derive(Deserialize)]
struct RuntimeConfigUpdate {
    #[serde(rename = "lowResourceMode")]
    low_resource_mode: bool,
    #[serde(rename = "ingestionEnabled")]
    ingestion_enabled: bool,
    #[serde(rename = "syncIntervalSec")]
    sync_interval_sec: u32,
}

#[tauri::command]
fn store_list_providers() -> Vec<StoreProvider> {
    store_providers()
}

#[tauri::command]
fn store_list_items(
    state: State<RuntimeState>,
    query: Option<String>,
    provider: Option<String>,
) -> Result<Vec<StoreItem>, String> {
    let query_normalized = query.unwrap_or_default().to_lowercase();
    let provider_normalized = provider.unwrap_or_default().to_lowercase();

    let catalog = state
        .store_catalog
        .lock()
        .map_err(|_| "store catalog lock poisoned".to_string())?;

    Ok(filter_catalog(
        &catalog,
        query_normalized.as_str(),
        provider_normalized.as_str(),
    ))
}

#[tauri::command]
fn store_get_item(state: State<RuntimeState>, item: String) -> Result<Option<StoreItem>, String> {
    let catalog = state
        .store_catalog
        .lock()
        .map_err(|_| "store catalog lock poisoned".to_string())?;
    Ok(catalog.iter().find(|entry| entry.id == item).cloned())
}

#[tauri::command]
fn store_refresh_cache(
    app: AppHandle,
    state: State<RuntimeState>,
) -> Result<StoreRefreshResult, String> {
    let providers = store_providers().len();
    let mut catalog = state
        .store_catalog
        .lock()
        .map_err(|_| "store catalog lock poisoned".to_string())?;
    *catalog = build_store_catalog();

    let result = StoreRefreshResult {
        items: catalog.len(),
        providers,
    };

    app.emit("store-refreshed", result.clone())
        .map_err(|e| format!("event emit failed: {e}"))?;

    Ok(result)
}

#[tauri::command]
fn library_list_apps(state: State<RuntimeState>) -> Result<Vec<LibraryApp>, String> {
    let library = state
        .library_apps
        .lock()
        .map_err(|_| "library lock poisoned".to_string())?;
    Ok(library.clone())
}

#[tauri::command]
fn library_save_item(
    app: AppHandle,
    state: State<RuntimeState>,
    item: String,
) -> Result<Option<LibraryApp>, String> {
    let catalog = state
        .store_catalog
        .lock()
        .map_err(|_| "store catalog lock poisoned".to_string())?;
    let store_item = catalog.iter().find(|entry| entry.id == item).cloned();
    drop(catalog);

    let Some(found) = store_item else {
        return Ok(None);
    };

    let mut library = state
        .library_apps
        .lock()
        .map_err(|_| "library lock poisoned".to_string())?;
    let Some(entry) = library_save_from_catalog(&found, &mut library) else {
        return Ok(None);
    };
    app.emit("library-updated", ())
        .map_err(|e| format!("event emit failed: {e}"))?;

    Ok(Some(entry))
}

#[tauri::command]
fn library_launch_item(
    app: AppHandle,
    state: State<RuntimeState>,
    item: String,
) -> Result<Option<LibraryApp>, String> {
    let mut library = state
        .library_apps
        .lock()
        .map_err(|_| "library lock poisoned".to_string())?;

    let result = library_launch(&mut library, &item, now_stamp().as_str());
    let Some(result) = result else {
        return Ok(None);
    };

    app.emit("library-updated", ())
        .map_err(|e| format!("event emit failed: {e}"))?;

    Ok(Some(result))
}

#[tauri::command]
fn library_remove_item(
    app: AppHandle,
    state: State<RuntimeState>,
    item: String,
) -> Result<bool, String> {
    let mut library = state
        .library_apps
        .lock()
        .map_err(|_| "library lock poisoned".to_string())?;

    let removed = library_remove(&mut library, &item);

    if removed {
        app.emit("library-updated", ())
            .map_err(|e| format!("event emit failed: {e}"))?;
    }

    Ok(removed)
}

#[tauri::command]
fn runtime_get_config(state: State<RuntimeState>) -> Result<RuntimeOverview, String> {
    let config = state
        .runtime_config
        .lock()
        .map_err(|_| "runtime config lock poisoned".to_string())?
        .clone();

    let library = state
        .library_apps
        .lock()
        .map_err(|_| "library lock poisoned".to_string())?;

    Ok(runtime_build_overview(&config, &library))
}

#[tauri::command]
fn runtime_update_config(
    state: State<RuntimeState>,
    payload: RuntimeConfigUpdate,
) -> Result<RuntimeOverview, String> {
    let mut config = state
        .runtime_config
        .lock()
        .map_err(|_| "runtime config lock poisoned".to_string())?;

    runtime_apply_update(&mut config, &payload);

    drop(config);
    runtime_get_config(state)
}

fn now_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    secs.to_string()
}

fn library_save_from_catalog(
    item: &StoreItem,
    library: &mut Vec<LibraryApp>,
) -> Option<LibraryApp> {
    if let Some(existing) = library.iter().find(|entry| entry.id == item.id).cloned() {
        return Some(existing);
    }

    let entry = LibraryApp {
        id: item.id.clone(),
        name: item.name.clone(),
        category: item.category.clone(),
        provider_name: item.provider_name.clone(),
        version: "1.0.0".to_string(),
        state: "ready".to_string(),
        last_launched: None,
    };

    library.push(entry.clone());
    Some(entry)
}

fn library_launch(library: &mut [LibraryApp], item_id: &str, stamp: &str) -> Option<LibraryApp> {
    let entry = library.iter_mut().find(|entry| entry.id == item_id)?;
    entry.state = "running".to_string();
    entry.last_launched = Some(stamp.to_string());
    Some(entry.clone())
}

fn library_remove(library: &mut Vec<LibraryApp>, item_id: &str) -> bool {
    let before = library.len();
    library.retain(|entry| entry.id != item_id);
    before != library.len()
}

fn runtime_apply_update(config: &mut RuntimeConfig, payload: &RuntimeConfigUpdate) {
    config.low_resource_mode = payload.low_resource_mode;
    config.ingestion_enabled = payload.ingestion_enabled;
    config.sync_interval_sec = payload.sync_interval_sec.clamp(5, 300);
}

fn runtime_build_overview(config: &RuntimeConfig, library: &[LibraryApp]) -> RuntimeOverview {
    RuntimeOverview {
        low_resource_mode: config.low_resource_mode,
        ingestion_enabled: config.ingestion_enabled,
        sync_interval_sec: config.sync_interval_sec,
        library_count: library.len(),
        running_count: library
            .iter()
            .filter(|entry| entry.state == "running")
            .count(),
    }
}

fn store_providers() -> Vec<StoreProvider> {
    let adapters = adapters::configured_adapters();
    adapters::providers_from_adapters(&adapters)
}

fn build_store_catalog() -> Vec<StoreItem> {
    let adapters = adapters::configured_adapters();
    adapters::catalog_from_adapters(&adapters)
}

fn filter_catalog(catalog: &[StoreItem], query: &str, provider: &str) -> Vec<StoreItem> {
    let mut filtered: Vec<StoreItem> = catalog
        .iter()
        .filter(|item| {
            (provider.is_empty() || item.provider_id == provider)
                && (query.is_empty()
                    || item.name.to_lowercase().contains(query)
                    || item.category.to_lowercase().contains(query)
                    || item.provider_name.to_lowercase().contains(query))
        })
        .cloned()
        .collect();

    filtered.sort_by(|a, b| b.rating.total_cmp(&a.rating));
    filtered
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RuntimeState::default())
        .invoke_handler(tauri::generate_handler![
            store_list_providers,
            store_list_items,
            store_get_item,
            store_refresh_cache,
            library_list_apps,
            library_save_item,
            library_launch_item,
            library_remove_item,
            runtime_get_config,
            runtime_update_config
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ed-forge");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn builds_unique_catalog_ids() {
        let catalog = build_store_catalog();
        let mut ids = HashSet::new();
        for item in &catalog {
            assert!(ids.insert(item.id.clone()));
        }
        assert_eq!(catalog.len(), 6);
    }

    #[test]
    fn filters_by_provider_and_query() {
        let catalog = build_store_catalog();
        let filtered = filter_catalog(&catalog, "module", "porofessor");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "porofessor-rune-assist");
    }

    #[test]
    fn adapter_registry_contains_expected_sources() {
        let providers = store_providers();
        assert_eq!(providers.len(), 2);
        assert!(providers.iter().any(|p| p.id == "tftmeta"));
        assert!(providers.iter().any(|p| p.id == "porofessor"));
    }

    #[test]
    fn library_save_launch_remove_lifecycle() {
        let catalog = build_store_catalog();
        let item = catalog
            .iter()
            .find(|entry| entry.id == "tftmeta-comp-scout")
            .expect("fixture exists");
        let mut library = Vec::new();

        let saved = library_save_from_catalog(item, &mut library).expect("saved");
        assert_eq!(saved.state, "ready");
        assert_eq!(library.len(), 1);

        let duplicate = library_save_from_catalog(item, &mut library).expect("duplicate returns");
        assert_eq!(duplicate.id, saved.id);
        assert_eq!(library.len(), 1);

        let launched = library_launch(&mut library, &saved.id, "123").expect("launches");
        assert_eq!(launched.state, "running");
        assert_eq!(launched.last_launched.as_deref(), Some("123"));

        assert!(library_remove(&mut library, &saved.id));
        assert!(library.is_empty());
        assert!(!library_remove(&mut library, "missing"));
    }

    #[test]
    fn runtime_update_clamps_interval() {
        let mut config = RuntimeConfig {
            low_resource_mode: true,
            ingestion_enabled: false,
            sync_interval_sec: 30,
        };

        let payload = RuntimeConfigUpdate {
            low_resource_mode: false,
            ingestion_enabled: true,
            sync_interval_sec: 1,
        };
        runtime_apply_update(&mut config, &payload);
        assert!(!config.low_resource_mode);
        assert!(config.ingestion_enabled);
        assert_eq!(config.sync_interval_sec, 5);
    }
}
