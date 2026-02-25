use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct InputRecord {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct OutputRecord {
    pub input_address: String,
    pub matched_address: Option<String>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub confidence: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MapboxBatchResponse {
    pub batch: Vec<MapboxFeatureCollection>,
}

#[derive(Debug, Deserialize)]
pub struct MapboxFeatureCollection {
    #[serde(default)]
    pub features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
pub struct Feature {
    pub geometry: Geometry,
    pub properties: Properties,
}

#[derive(Debug, Deserialize)]
pub struct Geometry {
    pub coordinates: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Properties {
    pub full_address: Option<String>,
    pub confidence: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MapboxBatchRequest {
    pub queries: Vec<MapboxQuery>,
}

#[derive(Debug, Serialize)]
pub struct MapboxQuery {
    pub q: String,
}
