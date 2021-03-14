use rand_distr::{Distribution, Normal};

use core::models::{Image, Pixel};
use std::time::{Instant, Duration};

// see https://www.kaggle.com/andyxie/k-means-clustering-implementation-in-python
// see https://en.wikipedia.org/wiki/Color_difference
// see https://gist.github.com/ryancat/9972419b2a78f329ce3aebb7f1a09152

pub fn cluster(pixels: &Vec<Pixel>, total_clusters: usize, min_error: u32, min_iterations: usize, max_iterations: usize, max_time: Duration) -> Vec<Pixel> {
    // simple kmeans, but it is okay for our purposes
    if pixels.len() == 0 {
        return Vec::new();
    }

    let mut pixels: Vec<(u8, u8, u8)> = pixels.iter().map(|v| (v.red, v.green, v.blue)).collect();
    let pixels_f64: Vec<(f64, f64, f64)> = pixels.iter().map(|v| (v.0 as f64, v.1 as f64, v.2 as f64)).collect();
    pixels.sort();

    let mean_pixel = mean_pixel(&pixels_f64).expect("mean pixel should be present because there is at least one pixel in this image");
    let std_pixel = std_pixel(&pixels_f64).expect("std pixel should be present because there is at least one pixel in this image");

    let centers: Vec<(f64, f64, f64)> = random_normal_f64s_vec(total_clusters);
    let mut centers: Vec<(u8, u8, u8)> = centers.iter()
        .map(|v| pixel_add(pixel_mul(*v, std_pixel), mean_pixel))
        .map(|v| (v.0 as u8, v.1 as u8, v.2 as u8))
        .collect();

    let mut centers_old;
    let mut clusters: Vec<usize> = vec![0; pixels.len()];

    // sum of all point coordinates inside cluster:
    let mut cluster_sums = vec![(0 as u32, 0 as u32, 0 as u32, 0 as u32); total_clusters];

    let mut error = u32::MAX;
    let mut iteration = 0;

    let started_at = Instant::now();

    while ((error > min_error && iteration < max_iterations) || iteration < min_iterations) && (Instant::now() - started_at) < max_time {
        let mut prev_pixel = (0, 0, 0);
        let mut prev_cluster: i32 = -1;

        for i in 0..cluster_sums.len() {
            cluster_sums[i] = (0, 0, 0, 0);
        }

        for pixel_index in 0..pixels.len() {
            let pixel = pixels[pixel_index];

            // matches previous pixel?
            if pixel == prev_pixel && prev_cluster != -1 {
                clusters[pixel_index] = prev_cluster as usize;

                let prev = cluster_sums[prev_cluster as usize];
                cluster_sums[prev_cluster as usize] = (
                    prev.0 + pixel.0 as u32, 
                    prev.1 + pixel.1 as u32, 
                    prev.2 + pixel.2 as u32, 
                    prev.3 + 1
                );
                
                continue;
            }

            let mut closest = 0;
            let mut closest_distance = i32::MAX;

            // Measure the distance to every center
            for cluster in 0..total_clusters {
                let distance = distance_u8(pixel, centers[cluster]);

                if distance < closest_distance {
                    closest_distance = distance;
                    closest = cluster;
                }
            }

            clusters[pixel_index] = closest;

            prev_pixel = pixel;
            prev_cluster = closest as i32;

            let prev = cluster_sums[closest];
            cluster_sums[closest] = (prev.0 + pixel.0 as u32, prev.1 + pixel.1 as u32, prev.2 + pixel.2 as u32, prev.3 + 1);
        }

        centers_old = centers.clone();

        for cluster in 0..total_clusters {
            let entry = cluster_sums[cluster];
            centers[cluster] = if entry.3 == 0 {
                let rand = random_normal_f64s();
                (rand.0 as u8, rand.1 as u8, rand.2 as u8)
            } else {
                ((entry.0 / entry.3) as u8, (entry.1 / entry.3) as u8, (entry.2 / entry.3) as u8)
            };
        }

        let mut sum = 0;
        for center in 0..total_clusters {
            let prev_center = centers_old[center];
            let new_center = centers[center];
            
            sum += (prev_center.0 as i32 - new_center.0 as i32).pow(2) + 
                (prev_center.1 as i32 - new_center.1 as i32).pow(2) + 
                (prev_center.2 as i32 - new_center.2 as i32).pow(2);
        }
        error = sum as u32;
        
        iteration += 1;
    }

    centers.iter()
        .map(|v| Pixel::from_rgb(v.0, v.1, v.2))
        .collect()
}

pub fn reduce_colors(image: &Image, colors: &Vec<Pixel>) -> Image {
    let mut new_image = Image::new(image.width, image.height);

    for y in 0..image.height {
        for x in 0..image.width {
            let pixel = image.get_pixel(x, y);

            let mut closest_color = 0;
            let mut color_dist = None;

            for color_index in 0..colors.len() {
                let color = colors[color_index];
                let dist = distance(pixel_to_f64s(&color), pixel_to_f64s(&pixel));
                
                if color_dist == None || dist < color_dist.expect("color dist should be present, because we checked that it is not None") {
                    color_dist = Some(dist);
                    closest_color = color_index;
                }
            }

            new_image.set_pixel(x, y, colors[closest_color].clone());
        }
    }

    new_image
}

fn pixel_to_f64s(pixel: &Pixel) -> (f64, f64, f64) {
    (pixel.red as f64, pixel.green as f64, pixel.blue as f64)
}

fn distance_u8(a: (u8, u8, u8), b: (u8, u8, u8)) -> i32 {
    let drp2 = (a.0 as i32 - b.0 as i32).pow(2);
    let dgp2 = (a.1 as i32 - b.1 as i32).pow(2);
    let dbp2 = (a.2 as i32 - b.2 as i32).pow(2);

    let t = (a.0 as i32 + b.0 as i32) / 2;

    2 * drp2 + 4 * dgp2 + 3 * dbp2 + t * (drp2 - dbp2) / 256
}

fn distance(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    let drp2 = (a.0 - b.0).powi(2);
    let dgp2 = (a.1 - b.1).powi(2);
    let dbp2 = (a.2 - b.2).powi(2);

    let t = (a.0 + b.0) / 2.0;

    (2.0 * drp2 + 4.0 * dgp2 + 3.0 * dbp2 + t * (drp2 - dbp2) / 256.0).sqrt()
}

fn random_normal_f64s() -> (f64, f64, f64) {
    let normal = Normal::new(0.0, 1.0).expect("expected to create a normal distribution random correctly");
    (normal.sample(&mut rand::thread_rng()), normal.sample(&mut rand::thread_rng()), normal.sample(&mut rand::thread_rng()))
}

fn random_normal_f64s_vec(ns: usize) -> Vec<(f64, f64, f64)> {
    (0..ns).map(|_| random_normal_f64s()).collect()
}

// like np.mean, but for pixels!
fn mean_pixel(pixels: &Vec<(f64, f64, f64)>) -> Option<(f64, f64, f64)> {
    pixels.iter()
        .map(|v| v.clone())
        .reduce(|a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2))
        .map(|v| (
            v.0 as f64 / pixels.len() as f64, 
            v.1 as f64 / pixels.len() as f64,
            v.2 as f64 / pixels.len() as f64
        ))
}

// like np.std, but for pixels!
fn std_pixel(pixels: &Vec<(f64, f64, f64)>) -> Option<(f64, f64, f64)> {
    Some((
        std_for_f64s(&pixels.iter().map(|v| v.0 as f64).collect()),
        std_for_f64s(&pixels.iter().map(|v| v.1 as f64).collect()),
        std_for_f64s(&pixels.iter().map(|v| v.2 as f64).collect()),
    ))
}

fn std_for_f64s(ns: &Vec<f64>) -> f64 {
    let mean = ns.iter().sum::<f64>() / ns.len() as f64;

    (ns.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / ns.len() as f64).sqrt()
}

fn pixel_mul(a: (f64, f64, f64), b: (f64, f64, f64)) -> (f64, f64, f64) {
    (a.0 * b.0, a.1 * b.1, a.2 * b.2)
}

fn pixel_add(a: (f64, f64, f64), b: (f64, f64, f64)) -> (f64, f64, f64) {
    (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}

#[cfg(test)]
mod tests {
    use core::models::ImageReader;
    use std::fs::read;

    use crate::reader::GIFReader;

    use super::*;

    #[test]
    fn test_clustering_simple() {
        let image = &GIFReader::new().read(
            &read("assets/sunrise.gif").expect("failed to read test image")
            )
            .expect("failed to read test image")[0];

        let pixels = image.pixels.clone();
        let centers = cluster(&pixels, 20, 0, 10, 100, Duration::from_secs(10));

        let image = reduce_colors(&image, &centers);
        
        let sun_color_distance = distance(pixel_to_f64s(&image.get_pixel(590, 278)), (252.0, 161.0, 1.0));

        assert_eq!(centers.len(), 20);
        assert!(sun_color_distance < 200.0);
    }
}