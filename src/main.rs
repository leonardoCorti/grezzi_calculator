use clap::Parser;
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;

/// Command-line arguments for the CSV parser
#[derive(Parser, Debug)]
#[command(author = "Leonardo Corti <leonardo.filippo@ymail.com>", version = "0.1", about = "tmp")]
struct Cli {
    /// The input CSV file
    #[arg(short, long)]
    input: String,

    /// The output CSV file
    #[arg(short, long)]
    output: String,

    /// Comma-separated list of columns to save (1-based index)
    #[arg(short, long, default_value = "1,3")]
    columns: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Parse column indices
    let columns: Vec<usize> = cli
        .columns
        .split(',')
        .map(|s| s.trim().parse::<usize>().expect("Invalid column index"))
        .collect();

    // Process CSV file
    process_csv(&cli.input, &cli.output, &columns)?;

    Ok(())
}

fn process_csv(input_path: &str, output_path: &str, columns: &[usize]) -> Result<(), Box<dyn Error>> {
    // Open input and output files
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().delimiter(b';').from_reader(input_file);
    let output_file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(output_file);

    // Process records
    for result in rdr.records() {
        let record = result?;
        let selected_fields: Vec<&str> = columns
            .iter()
            .filter_map(|&col| record.get(col - 1)) // Convert 1-based to 0-based index
            .collect();
        wtr.write_record(&selected_fields)?;
    }

    wtr.flush()?;
    Ok(())
}
