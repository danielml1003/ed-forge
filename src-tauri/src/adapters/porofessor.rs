use crate::models::{StoreItem, StoreProvider};

pub struct PorofessorAdapter;

impl super::CatalogAdapter for PorofessorAdapter {
    fn provider(&self) -> StoreProvider {
        StoreProvider {
            id: "porofessor".to_string(),
            name: "Porofessor".to_string(),
            region: "Global".to_string(),
            source_url: "https://porofessor.gg".to_string(),
        }
    }

    fn fetch_raw(&self) -> Vec<StoreItem> {
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
}
