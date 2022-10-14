use number::UnipolarFloat;

/// Scale value into the provided integer range.
pub fn unit_float_to_range(start: u8, end: u8, value: UnipolarFloat) -> u8 {
    ((end - start) as f64 * value.val()) as u8 + start
}
