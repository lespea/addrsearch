mod mapbox;
mod models;

use clap::Parser;
use color_eyre::eyre::{eyre, Context, Result};
use csv::{Reader, Writer};
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

use crate::mapbox::MapboxClient;
use crate::models::{InputRecord, OutputRecord};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input CSV file containing addresses
    input: String,

    /// Output CSV file for geocoding results
    #[arg(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize color-eyre for better panic and error reports
    color_eyre::install()?;

    // Initialize tracing-subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    // Determine the output path
    let input_path = Path::new(&args.input);
    let output_path = match args.output {
        Some(o) => PathBuf::from(o),
        None => {
            let stem = input_path.file_stem().ok_or_else(|| eyre!("Invalid input file name"))?;
            let mut pb = input_path.to_path_buf();
            pb.set_file_name(format!("{}_enriched", stem.to_string_lossy()));
            pb.set_extension("csv");
            pb
        }
    };

    // Get the Mapbox token from the environment at runtime.
    let token = env::var("MAPBOX_TOKEN").context("MAPBOX_TOKEN environment variable not set")?;

    // Open the input file
    let mut rdr = match Reader::from_path(&args.input) {
        Ok(r) => r,
        Err(e) => {
            match e.kind() {
                csv::ErrorKind::Io(io_err) if io_err.kind() == io::ErrorKind::NotFound => {
                    error!("'{}' not found. Please provide a valid CSV with an 'address' column.", args.input);
                    return Ok(());
                }
                _ => return Err(e.into()),
            }
        }
    };

    let mut wrt = Writer::from_path(&output_path).with_context(|| format!("Failed to create output file at {}", output_path.display()))?;

    let records: Vec<InputRecord> = rdr
        .deserialize()
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to deserialize input records")?;

    info!(records_count = records.len(), input_file = %args.input, "Starting geocoding process");

    // Mapbox batch limit is 50 queries per request.
    let chunks = records.chunks(50);
    let client = MapboxClient::new(token);

    for (i, chunk) in chunks.enumerate() {
        let addresses: Vec<String> = chunk.iter().map(|r| r.address.clone()).collect();
        info!(batch_index = i, batch_size = addresses.len(), "Geocoding batch");

        match client.geocode_batch(&addresses).await {
            Ok(batch_results) => {
                for (input, result) in addresses.into_iter().zip(batch_results) {
                    let record = OutputRecord {
                        input_address: input,
                        matched_address: result.as_ref().map(|f| f.place_name.clone()),
                        longitude: result.as_ref().map(|f| f.geometry.coordinates[0]),
                        latitude: result.as_ref().map(|f| f.geometry.coordinates[1]),
                        accuracy: result.as_ref().and_then(|f| {
                            f.properties.as_ref().and_then(|p| p.accuracy.clone())
                        }),
                    };
                    wrt.serialize(record).context("Failed to serialize output record")?;
                }
            }
            Err(e) => {
                warn!(error = %e, "Error geocoding batch; writing empty results for this batch");
                for input in addresses {
                    wrt.serialize(OutputRecord {
                        input_address: input,
                        matched_address: None,
                        longitude: None,
                        latitude: None,
                        accuracy: None,
                    })?;
                }
            }
        }
        // Flush after each batch
        wrt.flush().context("Failed to flush output writer")?;
    }

    info!(output_file = %output_path.display(), "Geocoding complete");
    Ok(())
}
