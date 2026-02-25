mod mapbox;
mod models;

use clap::Parser;
use color_eyre::eyre::{Context, Result, eyre};
use csv::Writer;
use std::env;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::mapbox::MapboxClient;
use crate::models::{InputRecord, OutputRecord, Bbox};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input CSV file containing addresses
    input: String,

    /// Output CSV file for geocoding results
    #[arg(short, long)]
    output: Option<String>,

    /// Bounding box to constrain search (min_lat,min_lon,max_lat,max_lon)
    #[arg(short, long, default_value = "44.72540,-93.75217,45.35455,-92.84715")]
    bbox: Bbox,

    /// Assume the input CSV has no header row
    #[arg(long)]
    no_header: bool,

    /// The 0-based index of the column containing the address
    #[arg(short, long, default_value_t = 0)]
    column_index: usize,
}

const BATCH_SIZE: usize = 1000;

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
            let stem = input_path
                .file_stem()
                .ok_or_else(|| eyre!("Invalid input file name"))?;
            let mut pb = input_path.to_path_buf();
            pb.set_file_name(format!("{}_enriched", stem.to_string_lossy()));
            pb.set_extension("csv");
            pb
        }
    };

    // Get the Mapbox token from the environment at runtime.
    let token = env::var("MAPBOX_TOKEN").context("MAPBOX_TOKEN environment variable not set")?;

    // Open the input file
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(!args.no_header)
        .from_path(&args.input)
        .with_context(|| format!("Failed to open input file: {}", args.input))?;

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result.context("Failed to read CSV record")?;
        let address = record
            .get(args.column_index)
            .ok_or_else(|| eyre!("Column index {} out of bounds in CSV", args.column_index))?
            .to_string();
        records.push(InputRecord { address });
    }

    let mut wrt = Writer::from_path(&output_path)
        .with_context(|| format!("Failed to create output file at {}", output_path.display()))?;

    info!(records_count = records.len(), input_file = %args.input, "Starting geocoding process (v6)");

    let chunks = records.chunks(BATCH_SIZE);
    let client = MapboxClient::new(token);
    let api_bbox_string = args.bbox.to_mapbox_string();

    for (i, chunk) in chunks.enumerate() {
        let addresses: Vec<String> = chunk.iter().map(|r| r.address.clone()).collect();
        info!(
            batch_index = i,
            batch_size = addresses.len(),
            "Geocoding batch"
        );

        match client.geocode_batch(&addresses, Some(&api_bbox_string)).await {
            Ok(batch_results) => {
                for (input, result) in addresses.into_iter().zip(batch_results) {
                    let record = OutputRecord {
                        input_address: input,
                        matched_address: result
                            .as_ref()
                            .and_then(|f| f.properties.full_address.clone()),
                        longitude: result.as_ref().map(|f| f.geometry.coordinates[0]),
                        latitude: result.as_ref().map(|f| f.geometry.coordinates[1]),
                        confidence: result
                            .as_ref()
                            .and_then(|f| f.properties.confidence.clone()),
                    };
                    wrt.serialize(record)
                        .context("Failed to serialize output record")?;
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
                        confidence: None,
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
