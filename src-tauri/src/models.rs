use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct StoreProvider {
    pub id: String,
    pub name: String,
    pub region: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
}

#[derive(Serialize, Clone)]
pub struct StoreItem {
    pub id: String,
    pub name: String,
    pub category: String,
    #[serde(rename = "providerId")]
    pub provider_id: String,
    #[serde(rename = "providerName")]
    pub provider_name: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
    #[serde(rename = "priceUsd")]
    pub price_usd: f64,
    pub rating: f64,
    pub stock: u32,
}
