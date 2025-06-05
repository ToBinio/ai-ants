pub fn mean(data: &Vec<f32>) -> f32 {
    let sum: f32 = data.iter().sum();
    sum / data.len() as f32
}

pub fn stddev(data: &Vec<f32>, mean: f32) -> f32 {
    let sum: f32 = data
        .iter()
        .map(|value| (value - mean) * (value - mean))
        .sum();
    (sum / (data.len() as f32 - 1.)).sqrt()
}
