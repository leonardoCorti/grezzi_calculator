use clap::Parser;
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

use rayon::prelude::*;

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
    #[arg(short, long, default_value = "1,5")]
    identifiers_columns: String,
    
    /// Column containing the width
    #[arg(short, long, default_value = "3")]
    width_column: usize,

    /// Column containing the length
    #[arg(short, long, default_value = "2")]
    length_column: usize,

    /// Offset
    #[arg(long, default_value = "10")]
    offset: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Parse column indices
    let identifiers_columns: Vec<usize> = cli
        .identifiers_columns
        .split(',')
        .map(|s| s.trim().parse::<usize>().expect("Invalid column index"))
        .collect();

    let identifiers: HashMap<String,Vec<Unit>> = get_data(&cli.input, &identifiers_columns, cli.width_column, cli.length_column)?;

    identifiers.par_iter().map(|(k,v)|do_stuff(k,v))
        .for_each(|(k,v)| println!("{:?}{:?}",k,v));
    
    Ok(())
}

fn do_stuff<'a>(k: &'a str, v: &'a[Unit]) -> (&'a str,Vec<Unit>) {
    let new_unit = Unit { height: v[0].width, width: v[0].height };
    return (k,vec![new_unit]);
}


fn get_data(
    input_path: &str,
    columns: &[usize],
    width_column: usize,
    height_column: usize
) -> Result<HashMap<String,Vec<Unit>>, Box<dyn Error>> {
    // Open input file
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().delimiter(b';').from_reader(input_file);
    let mut results: HashMap<String,Vec<Unit>> = HashMap::new();
    
    // Process records
    for result in rdr.records() {
        let record = result?;
        let selected_fields: Vec<&str> = columns
            .iter()
            .filter_map(|&col| record.get(col - 1)) // Convert 1-based to 0-based index
            .collect();
        let width: f32 = record.get(width_column -1).expect("cannot access width").replace(",", ".").parse()?;
        let height: f32 = record.get(height_column -1).expect("cannot access height").replace(",", ".").parse()?;
        let current_unit: Unit = Unit { height, width };
        let identifier = selected_fields.join(",");
        match results.get_mut(&identifier) {
            Some(id_list) => {
                id_list.push(current_unit);
            }
            None => {
                    let mut id_list: Vec<Unit> = Vec::new();
                    id_list.push(current_unit);
                    results.insert(identifier, id_list);
                }
        }
    }
    return Ok(results);
}

#[derive(Debug)]
struct Unit{
    height: f32,
    width: f32,
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
