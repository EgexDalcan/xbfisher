use std::ops::Div;

/// Gets the digits amount of digits of value
pub fn n_decimals(value: f32, digits: usize) -> f32 {
    let dec = match value {
        v if v <= 0_f32 => digits,
        _ => digits.saturating_sub(value.abs().log10() as usize + 1),
    };
    let vals = format!("{:.*}", dec, value);
    vals.parse().unwrap_or_else(|_|{value})
}

/// Calculates the mean of the input vector.
pub fn vec_mean(v: &Vec<f32>) -> f32{
    let viter = v.iter();
    viter.clone().sum::<f32>() as f32 / viter.clone().len() as f32
}

/// Calculates the standard deviation of the input vector.
pub fn vec_mdev(v: &Vec<f32>) -> f32{
    let avg = vec_mean(v);
    let mut sum= 0.0;
    for i in v{
        sum = sum + (i - avg).powi(2);
    };
    sum.div(v.len() as f32).sqrt()
}