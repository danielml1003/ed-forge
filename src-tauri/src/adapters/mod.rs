use std::collections::HashSet;

use crate::models::{StoreItem, StoreProvider};

pub mod porofessor;
pub mod tftmeta;

pub trait CatalogAdapter: Send + Sync {
    fn provider(&self) -> StoreProvider;
    fn fetch_raw(&self) -> Vec<StoreItem>;

    fn normalize(&self, raw: Vec<StoreItem>) -> Vec<StoreItem> {
        raw
    }

    fn validate(&self, item: &StoreItem) -> bool {
        !item.id.is_empty() && !item.name.is_empty() && !item.provider_id.is_empty()
    }

    fn fetch_catalog(&self) -> Vec<StoreItem> {
        self.normalize(self.fetch_raw())
            .into_iter()
            .filter(|item| self.validate(item))
            .collect()
    }
}

pub fn configured_adapters() -> Vec<Box<dyn CatalogAdapter>> {
    vec![
        Box::new(tftmeta::TftMetaAdapter),
        Box::new(porofessor::PorofessorAdapter),
    ]
}

pub fn providers_from_adapters(adapters: &[Box<dyn CatalogAdapter>]) -> Vec<StoreProvider> {
    adapters.iter().map(|adapter| adapter.provider()).collect()
}

pub fn catalog_from_adapters(adapters: &[Box<dyn CatalogAdapter>]) -> Vec<StoreItem> {
    let mut ids = HashSet::new();
    let mut items = Vec::new();

    for adapter in adapters {
        for item in adapter.fetch_catalog() {
            if ids.insert(item.id.clone()) {
                items.push(item);
            }
        }
    }

    items
}
