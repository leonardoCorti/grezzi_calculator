use clap::Parser;
use std::collections::HashMap;
use std::error::Error;

use rayon::prelude::*;

use grezzi_calculator::*;

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

    /// Offset_max
    #[arg(long, default_value = "10")]
    offset_min: usize,
    
    /// Offset max
    #[arg(long, default_value = "25")]
    offset_max: usize,
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
