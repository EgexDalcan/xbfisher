pub fn n_decimals(value: f32, digits: usize) -> f32 {
    let dec = match value {
        v if v <= 0_f32 => digits,
        _ => digits.saturating_sub(value.abs().log10() as usize + 1),
    };
    let vals = format!("{:.*}", dec, value);
    vals.parse().unwrap_or_else(|_|{value})
}

// Add averaging algo.