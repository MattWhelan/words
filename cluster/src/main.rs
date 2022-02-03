use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time;

use ndarray::Array;
use strsim::levenshtein;

fn main() -> Result<(), anyhow::Error> {
    let file = File::open("/var/log/wifi.log")?;
    let buffered = BufReader::new(file);
    let lines: Vec<String> = buffered.lines()
        .filter_map(|res| res.ok())
        .collect();

    let input = &lines[0..100];

    let start = time::Instant::now();
    let mut medoids = kmedoids::first_k(11);
    let dissim = Array::from_shape_fn((input.len(), input.len()), |(i, j)| levenshtein(&input[i], &input[j]) as u32);

    let distances_done = time::Instant::now();
    println!("built dissimilarity matrix for {} elements", input.len());

    let (loss, assi, iter, _): (i64, _, _, _) = kmedoids::fasterpam(&dissim, &mut medoids, 10);
    let finished = time::Instant::now();


    dbg!(loss, iter, &assi);
    let clusters = input.iter().zip(assi.iter())
        .fold(HashMap::new(), |mut acc, (l, c)| {
            acc.entry(*c).or_insert(Vec::new()).push(l);
            acc
        });

    dbg!(clusters);

    let distance_matrix = distances_done.duration_since(start).as_millis();
    let faster_pam = finished.duration_since(distances_done).as_millis();
    dbg!(distance_matrix, faster_pam);
    Ok(())
}
