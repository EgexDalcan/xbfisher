pub fn n_decimals(value: f32, digits: usize) -> usize {
    match value {
        v if v <= 0_f32 => {println!("{v}"); digits},
        _ => {let a = digits.saturating_sub(value.abs().log10() as usize + 1); println!("{a}"); a},
    }
}