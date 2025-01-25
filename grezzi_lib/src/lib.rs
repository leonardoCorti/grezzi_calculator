use std::{collections::HashMap, error::Error, fs::File, ops::Range};
use rand::Rng;

use csv::ReaderBuilder;
use image::{Rgb, RgbImage};

#[derive(Debug,Clone)]
pub struct Unit{
    pub height: f32,
    pub width: f32,
}

impl Unit {
    fn get_area(&self, offset: &Range<f32>) -> Area {
        let o_min: f32 = offset.start;
        let o_max: f32 = offset.end;
        let x: f32 = self.width;
        let y: f32 = self.height;
        //top rectangle
        let top = Rectangle{ 
            top_left: (x,y+o_max),
            down_right: (x+o_max,y+o_min),
        };
        //bottom rectagngle
        let down = Rectangle{
            top_left: (x+o_min,y+o_max),
            down_right: (x+o_max,y), 
        };
        return Area { top , down };
    }
}

#[derive(Debug,Clone)]
struct Rectangle{
    top_left: (f32,f32),
    down_right: (f32,f32),
}

impl Rectangle {
    fn intersection(&self, other: &Rectangle) -> Option<Rectangle> {
        None
    }
}

#[derive(Debug,Clone)]
pub struct Area{
    top: Rectangle,
    down: Rectangle,
}

impl Area {
    fn intersection(&self, other: &Area) -> Option<Area> { //TODO: wrong
        let top_intersection = self.top.intersection(&other.top);
        let down_intersection = self.down.intersection(&other.down);

        match (top_intersection, down_intersection) {
            (Some(top), Some(down)) => Some(Area { top, down }),
            (Some(top), None) => Some(Area {
                top,
                down: self.down.clone(), // Default to self.down if no intersection
            }),
            (None, Some(down)) => Some(Area {
                top: self.top.clone(), // Default to self.top if no intersection
                down,
            }),
            (None, None) => None, // No intersection at all
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

pub fn clustering_lazy<'a>(identifier: &'a str, units: &'a[Unit], offset: &Range<f32>) -> (&'a str,Vec<Cluster>) {
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

pub fn get_image(clusters: &Vec<(&str,Vec<Cluster>)>, offset: &Range<f32> ) -> RgbImage {
    let img_width = 2000;
    let img_height = 2000;
    
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
    let scale = 1;

    let mut img = RgbImage::new(img_width, img_height);

    let mut rng = rand::thread_rng();

    for (_, clusters) in clusters {
        for cluster in clusters {
            for unit in &cluster.units {
                let scaled_width = unit.width * scale;
                let scaled_height = unit.height * scale;
                let scaled_offset = Range {
                    start: offset.start * scale,
                    end: offset.end * scale,
                };

                let area = Unit {
                    height: scaled_height,
                    width: scaled_width,
                }
                .get_area(&scaled_offset);

                let color = Rgb([
                    rng.gen_range(0..255) as u8,
                    rng.gen_range(0..255) as u8,
                    rng.gen_range(0..255) as u8,
                ]);

                draw_rectangle(&mut img, &area.top, &color, 0.5);
                draw_rectangle(&mut img, &area.down, &color, 0.5);

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

fn draw_rectangle(img: &mut RgbImage, rect: &Rectangle, color: &Rgb<u8>, alpha: f32) {
    let (x1, y1) = rect.top_left;
    let (x2, y2) = rect.down_right;

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
