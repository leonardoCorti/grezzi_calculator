#![allow(dead_code)]
use std::{collections::HashMap, error::Error, fs::File, ops::Range};

use csv::ReaderBuilder;

#[derive(Debug,Clone)]
pub struct Unit{
    pub height: f32,
    pub width: f32,
}

impl Unit {
    fn get_area(&self, _offset: &Range<usize>) -> Area {
    todo!()
    }
}

#[derive(Debug,Clone)]
struct Point{
    top_left: (f32,f32),
    down_right: (f32,f32),
}

#[derive(Debug,Clone)]
pub struct Area{
    top: (Point,Point),
    down: (Point,Point),
}

impl Area {
    fn intersection(&self, _other: &Area) -> Option<Area> {
        todo!()
    }
}

#[derive(Debug,Clone)]
pub struct Cluster {
    pub area: Area,
    pub units: Vec<Unit>,
}

pub fn get_data_from_csv(
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

pub fn clustering<'a>(identifier: &'a str, units: &'a[Unit], offset: &Range<usize>) -> (&'a str,Vec<Cluster>) {
    let mut clusters: Vec<Cluster> = Vec::new();

    for current_unit in units {
        let current_area = current_unit.get_area(offset);

        let matching_cluster = clusters.iter_mut()
            .filter_map(|c| {
                match c.area.intersection(&current_area) {
                    Some(area_of_intersection) => Some((c,area_of_intersection)),
                    None => None,
                }
            }).next();

        match matching_cluster {
            Some((the_cluster,area_of_intersection)) => { //update the matching cluster
                the_cluster.units.push(current_unit.clone());
                the_cluster.area = area_of_intersection;
            },
            None => { //create a new cluster
                let new_cluster: Cluster = Cluster { area: current_area, units: vec![current_unit.clone()] };
                clusters.push(new_cluster);
            },
        }
    }

    return (identifier,clusters);
}
