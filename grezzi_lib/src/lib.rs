use std::{collections::HashMap, error::Error, fs::File, ops::Range};
use rand::Rng;
use tracing::{debug, info, trace};

use csv::ReaderBuilder;
use image::{Rgb, RgbImage};

/// height is X and width is Y
#[derive(Debug,Clone)]
pub struct Unit{
    pub height: f32,
    pub width: f32,
}

impl Unit {

    /// put height in x and width in y
    fn new(x: f32, y: f32) -> Unit {
        return Unit { height: x, width: y};
    }

    fn get_area(&self, offset: &Range<f32>) -> Area {
        return Area { 
            top_left: Point { 
                x: self.height + offset.start, 
                y: self.width + offset.end, 
            }, 
            down_right: Point { 
                x: self.height + offset.end, 
                y: self.width + offset.start,
            } 
        };
    }
}

#[derive(Debug,Clone)]
struct Point{
    x: f32,
    y: f32,
}

#[derive(Debug,Clone)]
pub struct Area{
    top_left: Point,
    down_right: Point,
}

impl Area {
    #[tracing::instrument]
    fn intersection(&self, other: &Area) -> Option<Area> {
        // Compute the intersection points
        let top_left = Point {
            x: self.top_left.x.max(other.top_left.x),
            y: self.top_left.y.min(other.top_left.y),
        };
        let down_right = Point {
            x: self.down_right.x.min(other.down_right.x),
            y: self.down_right.y.max(other.down_right.y),
        };
        trace!("intersection points are {:?}, {:?}", top_left, down_right);

        // Check if the areas overlap
        if top_left.x <= down_right.x && top_left.y >= down_right.y {
            trace!("returned an area");
            Some(Area { top_left, down_right })
        } else {
            trace!("returned None");
            None
        }
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
        let current_unit: Unit = Unit::new(width, height);
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

pub fn clustering_lazy<'a>(
    identifier: &'a str,
    units: &'a[Unit],
    offset: &Range<f32>
) -> (&'a str,Vec<Cluster>) {
    let mut clusters: Vec<Cluster> = Vec::new();
    info!("lazy clustering init");

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
                debug!("updated the cluster {:?} with {:?}", the_cluster, current_unit);
                the_cluster.units.push(current_unit.clone());
                the_cluster.area = area_of_intersection;
            },
            None => { //create a new cluster
                debug!("made a mew cluster with {:?}", current_unit);
                let new_cluster: Cluster = Cluster { area: current_area, units: vec![current_unit.clone()] };
                clusters.push(new_cluster);
            },
        }
    }

    return (identifier,clusters);
}

pub fn get_image(clusters: &Vec<(&str,Vec<Cluster>)>, offset: &Range<f32> ) -> RgbImage {
    let img_width = 1000;
    let img_height = 1000;
    
    // Determine scaling factors to fit all units within the image

    let max_width = clusters
        .iter()
        .flat_map(|(_, cluster_list)| cluster_list.iter())
        .flat_map(|cluster| cluster.units.iter())
        .map(|u| u.width)
        .fold(0.0 / 0.0, f32::max);

    let max_height = clusters
        .iter()
        .flat_map(|(_, cluster_list)| cluster_list.iter())
        .flat_map(|cluster| cluster.units.iter())
        .map(|u| u.height)
        .fold(0.0 / 0.0, f32::max);

    let scale_x = img_width as f32 / (max_width + offset.end);
    let scale_y = img_height as f32 / (max_height + offset.end);
    let scale = 1.0;

    let mut img = RgbImage::new(img_width, img_height);

    let mut rng = rand::thread_rng();

    for (_, clusters) in clusters {
        for cluster in clusters {
            for unit in &cluster.units {
                let scaled_width = unit.width ;
                let scaled_height = unit.height;
                let area = cluster.area.clone();

                let color = Rgb([
                    rng.gen_range(0..255) as u8,
                    rng.gen_range(0..255) as u8,
                    rng.gen_range(0..255) as u8,
                ]);

                draw_rectangle(&mut img, &area, &color, 0.5);

                draw_circle(&mut img, unit.width as u32, unit.height as u32, 3, &color, 0.5);
            }
        }
    }

    return img;
}

fn blend_color(base: &Rgb<u8>, overlay: &Rgb<u8>, alpha: f32) -> Rgb<u8> {
    Rgb([
        ((1.0 - alpha) * base[0] as f32 + alpha * overlay[0] as f32) as u8,
        ((1.0 - alpha) * base[1] as f32 + alpha * overlay[1] as f32) as u8,
        ((1.0 - alpha) * base[2] as f32 + alpha * overlay[2] as f32) as u8,
    ])
}

fn draw_rectangle(img: &mut RgbImage, rect: &Area, color: &Rgb<u8>, alpha: f32) {
    let x1 = rect.top_left.x;
    let y1 = rect.top_left.y;
    let x2 = rect.down_right.x;
    let y2 = rect.down_right.y;

    for x in x1 as u32..x2 as u32 {
        for y in y2 as u32..y1 as u32 {
            if x < img.width() && y < img.height() {
                let base_color = img.get_pixel(x, img.height() - 1 - y);
                let blended_color = blend_color(base_color, color, alpha);
                img.put_pixel(x, img.height() - 1 - y, blended_color);
            }
        }
    }
}

fn draw_circle(img: &mut RgbImage, center_x: u32, center_y: u32, radius: u32, color: &Rgb<u8>, alpha: f32) {
    let img_height = img.height();
    for x in (center_x as i32 - radius as i32)..=(center_x as i32 + radius as i32) {
        for y in (center_y as i32 - radius as i32)..=(center_y as i32 + radius as i32) {
            let dx = x - center_x as i32;
            let dy = y - center_y as i32;
            if dx * dx + dy * dy <= (radius as i32).pow(2) {
                if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
                    let base_color = img.get_pixel(x as u32, img_height - 1 - y as u32);
                    let blended_color = blend_color(base_color, color, alpha);
                    img.put_pixel(x as u32, img_height - 1 - y as u32, blended_color);
                }
            }
        }
    }
}
