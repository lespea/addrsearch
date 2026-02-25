use crate::models::{Feature, MapboxBatchResponse, MapboxQuery};
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

    pub async fn geocode_batch(
        &self,
        addresses: &[String],
        bbox: Option<&str>,
    ) -> Result<Vec<Option<Feature>>> {
        if addresses.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Geocoding {} addresses using v6 API", addresses.len());

        let url = "https://api.mapbox.com/search/geocode/v6/batch";

        let queries: Vec<MapboxQuery> = addresses
            .iter()
            .map(|a| MapboxQuery {
                q: a.clone(),
                bbox: bbox.map(|s| s.to_string()),
                limit: Some(1),
            })
            .collect();

        let resp = self
            .client
            .post(url)
            .query(&[("access_token", &self.token)])
            .json(&queries)
            .send()
            .await
            .context("Failed to send Mapbox v6 API request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_else(|_| "unknown error".into());
            error!(%status, %err_text, "Mapbox v6 API request failed");
            return Err(anyhow::anyhow!(
                "API request failed with status: {}. Body: {}",
                status,
                err_text
            ));
        }

        let batch_response: MapboxBatchResponse = resp
            .json()
            .await
            .context("Failed to parse Mapbox v6 batch response as JSON")?;

        let results = batch_response
            .batch
            .into_iter()
            .map(|collection| collection.features.into_iter().next())
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::MapboxBatchResponse;

    #[test]
    fn test_deserialize_v6_batch_response() {
        let json_data = r#"{
            "batch": [
                {
                    "type": "FeatureCollection",
                    "features": [
                        {
                            "type": "Feature",
                            "id": "address.1",
                            "geometry": {
                                "type": "Point",
                                "coordinates": [-73.9857, 40.7484]
                            },
                            "properties": {
                                "full_address": "350 5th Ave, New York, NY 10118, United States",
                                "confidence": "exact"
                            }
                        }
                    ]
                },
                {
                    "type": "FeatureCollection",
                    "features": [
                        {
                            "type": "Feature",
                            "id": "address.2",
                            "geometry": {
                                "type": "Point",
                                "coordinates": [-122.4194, 37.7749]
                            },
                            "properties": {
                                "full_address": "Market St, San Francisco, CA 94103, United States",
                                "confidence": "high"
                            }
                        }
                    ]
                }
            ]
        }"#;

        let response: MapboxBatchResponse = serde_json::from_str(json_data).unwrap();
        assert_eq!(response.batch.len(), 2);

        let res1 = &response.batch[0].features[0];
        assert_eq!(res1.geometry.coordinates, vec![-73.9857, 40.7484]);
        assert_eq!(
            res1.properties.full_address.as_deref(),
            Some("350 5th Ave, New York, NY 10118, United States")
        );

        let res2 = &response.batch[1].features[0];
        assert_eq!(res2.geometry.coordinates, vec![-122.4194, 37.7749]);
        assert_eq!(
            res2.properties.full_address.as_deref(),
            Some("Market St, San Francisco, CA 94103, United States")
        );
    }
}
