use crate::models::{StoreItem, StoreProvider};

pub struct TftMetaAdapter;

impl super::CatalogAdapter for TftMetaAdapter {
    fn provider(&self) -> StoreProvider {
        StoreProvider {
            id: "tftmeta".to_string(),
            name: "TFTMeta".to_string(),
            region: "Global".to_string(),
            source_url: "https://tftmeta.gg".to_string(),
        }
    }

    fn fetch_raw(&self) -> Vec<StoreItem> {
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
}
