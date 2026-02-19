use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

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
struct StoreProvider {
    id: String,
    name: String,
    region: String,
    #[serde(rename = "sourceUrl")]
    source_url: String,
}

#[derive(Serialize, Clone)]
struct StoreItem {
    id: String,
    name: String,
    category: String,
    #[serde(rename = "providerId")]
    provider_id: String,
    #[serde(rename = "providerName")]
    provider_name: String,
    #[serde(rename = "sourceUrl")]
    source_url: String,
    #[serde(rename = "priceUsd")]
    price_usd: f64,
    rating: f64,
    stock: u32,
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

    if let Some(existing) = library.iter().find(|entry| entry.id == found.id).cloned() {
        return Ok(Some(existing));
    }

    let entry = LibraryApp {
        id: found.id,
        name: found.name,
        category: found.category,
        provider_name: found.provider_name,
        version: "1.0.0".to_string(),
        state: "ready".to_string(),
        last_launched: None,
    };

    library.push(entry.clone());
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

    let Some(entry) = library.iter_mut().find(|entry| entry.id == item) else {
        return Ok(None);
    };

    entry.state = "running".to_string();
    entry.last_launched = Some(now_stamp());
    let result = entry.clone();

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

    let before = library.len();
    library.retain(|entry| entry.id != item);
    let removed = library.len() != before;

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

    Ok(RuntimeOverview {
        low_resource_mode: config.low_resource_mode,
        ingestion_enabled: config.ingestion_enabled,
        sync_interval_sec: config.sync_interval_sec,
        library_count: library.len(),
        running_count: library
            .iter()
            .filter(|entry| entry.state == "running")
            .count(),
    })
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

    config.low_resource_mode = payload.low_resource_mode;
    config.ingestion_enabled = payload.ingestion_enabled;
    config.sync_interval_sec = payload.sync_interval_sec.clamp(5, 300);

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

fn store_providers() -> Vec<StoreProvider> {
    vec![
        StoreProvider {
            id: "tftmeta".to_string(),
            name: "TFTMeta".to_string(),
            region: "Global".to_string(),
            source_url: "https://tftmeta.gg".to_string(),
        },
        StoreProvider {
            id: "porofessor".to_string(),
            name: "Porofessor".to_string(),
            region: "Global".to_string(),
            source_url: "https://porofessor.gg".to_string(),
        },
    ]
}

fn tftmeta_items() -> Vec<StoreItem> {
    vec![
        StoreItem {
            id: "tftmeta-comp-scout".to_string(),
            name: "Comp Scout Pack".to_string(),
            category: "Team Comp".to_string(),
            provider_id: "tftmeta".to_string(),
            provider_name: "TFTMeta".to_string(),
            source_url: "https://tftmeta.gg".to_string(),
            price_usd: 2.49,
            rating: 4.7,
            stock: 150,
        },
        StoreItem {
            id: "tftmeta-trait-tracker".to_string(),
            name: "Trait Tracker Widget".to_string(),
            category: "Overlay Widget".to_string(),
            provider_id: "tftmeta".to_string(),
            provider_name: "TFTMeta".to_string(),
            source_url: "https://tftmeta.gg".to_string(),
            price_usd: 3.99,
            rating: 4.5,
            stock: 80,
        },
        StoreItem {
            id: "tftmeta-level-timer".to_string(),
            name: "Level Timer HUD".to_string(),
            category: "HUD".to_string(),
            provider_id: "tftmeta".to_string(),
            provider_name: "TFTMeta".to_string(),
            source_url: "https://tftmeta.gg".to_string(),
            price_usd: 1.99,
            rating: 4.2,
            stock: 210,
        },
    ]
}

fn porofessor_items() -> Vec<StoreItem> {
    vec![
        StoreItem {
            id: "porofessor-match-insight".to_string(),
            name: "Match Insight Panel".to_string(),
            category: "Analysis".to_string(),
            provider_id: "porofessor".to_string(),
            provider_name: "Porofessor".to_string(),
            source_url: "https://porofessor.gg".to_string(),
            price_usd: 4.99,
            rating: 4.8,
            stock: 95,
        },
        StoreItem {
            id: "porofessor-rune-assist".to_string(),
            name: "Rune Assist Module".to_string(),
            category: "Assistant".to_string(),
            provider_id: "porofessor".to_string(),
            provider_name: "Porofessor".to_string(),
            source_url: "https://porofessor.gg".to_string(),
            price_usd: 2.99,
            rating: 4.4,
            stock: 180,
        },
        StoreItem {
            id: "porofessor-session-recap".to_string(),
            name: "Session Recap Exporter".to_string(),
            category: "Reporting".to_string(),
            provider_id: "porofessor".to_string(),
            provider_name: "Porofessor".to_string(),
            source_url: "https://porofessor.gg".to_string(),
            price_usd: 5.49,
            rating: 4.6,
            stock: 70,
        },
    ]
}

fn build_store_catalog() -> Vec<StoreItem> {
    let mut catalog = Vec::new();
    let mut ids = HashSet::new();

    for entry in tftmeta_items().into_iter().chain(porofessor_items()) {
        if ids.insert(entry.id.clone()) {
            catalog.push(entry);
        }
    }

    catalog
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
}
