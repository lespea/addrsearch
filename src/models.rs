use serde::{Deserialize, Serialize};

use anyhow::anyhow;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Bbox {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

impl Bbox {
    pub fn to_mapbox_string(&self) -> String {
        format!("{},{},{},{}", self.min_lon, self.min_lat, self.max_lon, self.max_lat)
    }
}

impl FromStr for Bbox {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<f64> = s
            .split(',')
            .map(|p| p.trim().parse::<f64>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Failed to parse coordinate: {}", e))?;

        if parts.len() != 4 {
            return Err(anyhow!("Bbox must have 4 coordinates: min_lon,min_lat,max_lon,max_lat"));
        }

        Ok(Bbox {
            min_lon: parts[0],
            min_lat: parts[1],
            max_lon: parts[2],
            max_lat: parts[3],
        })
    }
}


#[derive(Debug)]
pub struct InputRecord {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct OutputRecord {
    pub input_address: String,
    pub matched_address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
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
    pub match_code: Option<MatchCode>,
}

#[derive(Debug, Deserialize)]
pub struct MatchCode {
    pub confidence: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MapboxQuery {
    pub q: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}
