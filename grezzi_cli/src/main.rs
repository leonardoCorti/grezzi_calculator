use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};

use rayon::prelude::*;

use grezzi_lib::*;

/// program to read and calculate the grezzi dimensions
#[derive(Parser, Debug)]
#[command(styles=get_styles())]
struct Cli {
    /// The input CSV file
    input: String,

    /// The output CSV file
    #[arg(short, long)]
    output: Option<String>,

    /// Comma-separated list of columns with important identifiers (1-based index)
    #[arg(short, long, default_value = "5")]
    identifiers_columns: String,
    
    /// Column containing the width
    #[arg(short, long, default_value = "3")]
    width_column: usize,

    /// Column containing the length
    #[arg(short, long, default_value = "2")]
    length_column: usize,

    /// Offset_max
    #[arg(long, default_value = "10")]
    offset_min: f32,
    
    /// Offset max
    #[arg(long, default_value = "25")]
    offset_max: f32,

    /// create an image representing the distributions
    #[arg(short,long, default_value="false")]
    plot: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Parse column indices
    let identifiers_columns: Vec<usize> = cli
        .identifiers_columns
        .split(',')
        .map(|s| s.trim().parse::<usize>().expect("Invalid column index"))
        .collect();

    let identifiers: HashMap<String,Vec<Unit>> = get_data_from_csv(&cli.input, &identifiers_columns, cli.width_column, cli.length_column)?;

    let offsets = cli.offset_min..cli.offset_max;

    let mut writer: Box<dyn Write> = if let Some(ref output) = &cli.output {
        // If output is Some, write to the specified file
        Box::new(File::create(output).expect("Failed to create output file"))
    } else {
        // If output is None, write to stdout
        Box::new(io::stdout())
    };

    let clusters: Vec<_> = identifiers.par_iter().map(|(k,v)|clustering_lazy(k,v, &offsets)).collect();

    clusters.iter().for_each(|(k,v)| {
        writeln!(writer, "{:?}{:#?}", k, v).expect("Failed to write to output file");
    });

    if cli.plot {
        let plot = get_image(&clusters, &offsets);
        plot.save("plot.png")?;
    }

    Ok(())
}

//styling for help flag
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))),
        )
}
