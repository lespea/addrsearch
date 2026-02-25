use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct InputRecord {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct OutputRecord {
    pub input_address: String,
    pub matched_address: Option<String>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub accuracy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MapboxFeatureCollection {
    #[serde(default)]
    pub features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
pub struct Feature {
    pub place_name: String,
    pub geometry: Geometry,
    pub properties: Option<Properties>,
}

#[derive(Debug, Deserialize)]
pub struct Geometry {
    pub coordinates: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Properties {
    pub accuracy: Option<String>,
}
