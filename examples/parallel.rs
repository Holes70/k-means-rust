use rand::{thread_rng, Rng};
use std::ops::{Div, Add, AddAssign};
use std::fs::File;
use std::io::{BufRead, BufReader};
use plotters::prelude::*;
use rayon::{current_num_threads, prelude::*};
use std::time::Instant;
use std::sync::{Mutex};
use std::cell::RefCell;

const COUNT_OF_CLUSTERS:usize = 55;
const MATRIX: Matrix = Matrix { x: 10.0, y: 10.0 };
const NUM_OF_CPU_CORES: &str = "1";
const DRAW_CENTRAOIDS: bool = false; 

struct Matrix {
  x: f64,
  y: f64
}

#[derive(Debug, Copy, Clone)]
struct Point {
  x: f64,
  y: f64,
}

impl Point {
  fn distance(&self, other: &Point) -> f64 {
    // sqrt((x2 - x1)^2 + (y2 - y1)^2)
    ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
  }
}

impl Div<f64> for Point {
  type Output =  Point;

  fn div(self, rhs: f64) -> Point {
    Point {
      x: self.x / rhs,
      y: self.y / rhs,
    }
  }
}

impl Add<Point> for Point {
  type Output = Point; 

  fn add(self, other: Point) -> Point {
    Point {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }
}

impl AddAssign<Point> for Point {
  fn add_assign(&mut self, other: Point) {
    *self = *self + other;
  }
}

impl PartialEq<Point> for Point {
  fn eq(&self, other: &Point) -> bool {
    self.x == other.x && self.y == other.y
  }
}

fn read_points() -> Vec<Point> {
  let file = File::open("inputs/points.txt").unwrap();
  let reader = BufReader::new(file);

  let mut points = Vec::new();
  for line in reader.lines() {
    let line = line.unwrap();
    let coords: Vec<f64> = line
      .split_whitespace()
      .map(|coord| coord.parse().unwrap())
      .collect();
    
    let point = Point {
      x: coords[0], 
      y: coords[1]
    };

    points.push(point);
  }

  points
}

fn generate_colors(count_of_clusters: usize) -> Vec<RGBColor> {
  let mut colors = vec![];
  let mut rng = thread_rng();

  for _i in 0..count_of_clusters {
    colors.push(RGBColor(rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255)));
  }
  
  colors
}

fn k_means(points: &Vec<Point>, cluster_count: usize) -> (Vec<Point>, Vec<Vec<Point>>) {
  let mut rng = thread_rng();

  // Init some random Points of centroids
  let mut centroids: Vec<Point> = (0..cluster_count)
    .map(|_| Point {
      x: rng.gen_range(0.0..MATRIX.x),
      y: rng.gen_range(0.0..MATRIX.y),
    })
    .collect::<Vec<Point>>();

  let mut clusters_glob: Vec<Vec<Point>> = vec![];
  let mut centroids_glob: Vec<Point> = vec![];

  loop {    
    let _clusters_mutex = Mutex::new(RefCell::new(vec![Vec::new(); cluster_count]));

    points.par_iter().for_each(|point| {
      let mut min_distance = std::f64::INFINITY;
      let mut closest_centroid = 0;

      // Check all centroids and calculate which is closest
      for (i, centroid) in centroids.iter().enumerate() {
        let distance = point.distance(&centroid);

        if distance < min_distance {
          min_distance = distance;
          closest_centroid = i;
        }
      }

      // Unlock _clusters and push new value
      let clusters = _clusters_mutex.lock().unwrap();
      clusters.borrow_mut()[closest_centroid].push(point.clone());
    });

    // Insert cluster to global variable
    let clusters = _clusters_mutex.lock().unwrap();
    //clusters_glob = clusters.borrow_mut().to_vec();

    // Calculate centroids for each cluster by SUM(clusters) / COUNT(clusters)
    for (i, cluster) in clusters.borrow_mut().iter().enumerate() {
      let centroid = cluster.par_iter()
        .map(|&point| point)
        .reduce(|| Point { x: 0.0, y: 0.0 }, |acc, point| acc + point) / cluster.len() as f64;

      centroids[i] = centroid;
    }

    if centroids_glob == centroids {
      break;
    }

    centroids_glob = centroids.clone();
  }

  (centroids, clusters_glob)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  std::env::set_var("RAYON_NUM_THREADS", NUM_OF_CPU_CORES);
  
  let points = read_points();

  let k_means_calculate_start = Instant::now();
  let (centroids, clusters) = k_means(&points, COUNT_OF_CLUSTERS);
  let k_means_calculate_end = Instant::now();

  // Create plot
  let root = BitMapBackend::new("outputs/k-means.png", (800, 600))
    .into_drawing_area();

  root.fill(&WHITE)?;

  // Set margin and X, Y width
  let mut chart = ChartBuilder::on(&root)
    .margin(10)
    .build_cartesian_2d(0.0..MATRIX.x, 0.0..MATRIX.y)?;

  if DRAW_CENTRAOIDS {
    let centroid_circles: Vec<_> = centroids
      .into_iter()
      .map(|p| Circle::new((p.x, p.y), 2, Into::<RGBColor>::into(RED).filled()))
      .collect();

    chart.draw_series(centroid_circles)?;
  }

  let colors = generate_colors(COUNT_OF_CLUSTERS);

  for (index, cluster) in clusters.iter().enumerate() {
    let color = colors[index];

    let cluster_to_draw: Vec<_> = cluster.clone()
      .into_par_iter()
      .map(|p| Circle::new((p.x, p.y), 1, Into::<RGBColor>::into(color).filled()))
      .collect();

    chart.draw_series(cluster_to_draw)?;
  }

  println!("K-means time consumed: {:?}", k_means_calculate_end - k_means_calculate_start);
  println!("CPU cores used: {}", current_num_threads());

  Ok(())
}
