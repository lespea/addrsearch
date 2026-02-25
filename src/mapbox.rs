use crate::models::{Feature, MapboxFeatureCollection};
use anyhow::{Context, Result};
use reqwest::Client;
use tracing::{debug, error};

pub struct MapboxClient {
    client: Client,
    token: String,
}

impl MapboxClient {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    pub async fn geocode_batch(&self, addresses: &[String]) -> Result<Vec<Option<Feature>>> {
        // Mapbox v5 Batch Geocoding uses semicolon-separated queries.
        let encoded_queries: Vec<String> = addresses
            .iter()
            .map(|s| urlencoding::encode(s).into_owned())
            .collect();

        let query_string = encoded_queries.join(";");

        debug!("Geocoding {} addresses", addresses.len());

        // Note: Using mapbox.places-permanent for batch geocoding as per docs.
        let url = format!(
            "https://api.mapbox.com/geocoding/v5/mapbox.places-permanent/{}.json",
            query_string
        );

        let resp = self
            .client
            .get(&url)
            .query(&[("access_token", &self.token)])
            .send()
            .await
            .context("Failed to send Mapbox API request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_else(|_| "unknown error".into());
            error!(%status, %err_text, "Mapbox API request failed");
            return Err(anyhow::anyhow!(
                "API request failed with status: {}. Body: {}",
                status,
                err_text
            ));
        }

        let json_val: serde_json::Value = resp.json().await.context("Failed to parse Mapbox response as JSON")?;
        let mut batch_results = Vec::new();

        if let Some(arr) = json_val.as_array() {
            for item in arr {
                let collection: MapboxFeatureCollection = serde_json::from_value(item.clone())
                    .context("Failed to deserialize Mapbox feature collection")?;
                batch_results.push(collection.features.into_iter().next());
            }
        } else {
            // Single result (if only one query was sent)
            let collection: MapboxFeatureCollection = serde_json::from_value(json_val)
                .context("Failed to deserialize single Mapbox feature collection")?;
            batch_results.push(collection.features.into_iter().next());
        }

        Ok(batch_results)
    }
}
