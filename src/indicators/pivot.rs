/// Detects pivot highs in a price series
/// Returns Some(high_price) if a pivot is found at the lookback position, None otherwise
pub fn pivot_high(highs: &[f64], left_bars: usize, right_bars: usize) -> Option<f64> {
    let total_required = left_bars + right_bars + 1;

    if highs.len() < total_required {
        return None;
    }

    let pivot_index = highs.len() - right_bars - 1;
    let pivot_value = highs[pivot_index];

    for i in (pivot_index - left_bars)..pivot_index {
        if highs[i] > pivot_value {
            return None;
        }
    }

    for i in (pivot_index + 1)..=(pivot_index + right_bars) {
        if highs[i] > pivot_value {
            return None;
        }
    }

    Some(pivot_value)
}

/// Detects pivot lows in a price series
/// Returns Some(low_price) if a pivot is found at the lookback position, None otherwise
pub fn pivot_low(lows: &[f64], left_bars: usize, right_bars: usize) -> Option<f64> {
    let total_required = left_bars + right_bars + 1;

    if lows.len() < total_required {
        return None;
    }

    let pivot_index = lows.len() - right_bars - 1;
    let pivot_value = lows[pivot_index];

    for i in (pivot_index - left_bars)..pivot_index {
        if lows[i] < pivot_value {
            return None;
        }
    }

    for i in (pivot_index + 1)..=(pivot_index + right_bars) {
        if lows[i] < pivot_value {
            return None;
        }
    }

    Some(pivot_value)
}

/// Maintains a dynamic center line from detected pivot points
pub struct PivotCenterLine {
    center: Option<f64>,
}

impl PivotCenterLine {
    pub fn new() -> Self {
        Self { center: None }
    }

    pub fn update(&mut self, pivot_price: f64) {
        self.center = Some(match self.center {
            None => pivot_price,
            Some(prev) => (prev * 2.0 + pivot_price) / 3.0,
        });
    }

    pub fn get(&self) -> Option<f64> {
        self.center
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pivot_high() {
        // Test data: [100, 105, 110, 105, 100]
        // Pivot should be at index 2 (110) with left=1, right=2
        let highs = vec![100.0, 105.0, 110.0, 105.0, 100.0];
        let result = pivot_high(&highs, 1, 2);
        assert_eq!(result, Some(110.0));
    }

    #[test]
    fn test_pivot_low() {
        // Test data: [100, 95, 90, 95, 100]
        // Pivot should be at index 2 (90) with left=1, right=2
        let lows = vec![100.0, 95.0, 90.0, 95.0, 100.0];
        let result = pivot_low(&lows, 1, 2);
        assert_eq!(result, Some(90.0));
    }

    #[test]
    fn test_pivot_center_line() {
        let mut center = PivotCenterLine::new();
        assert_eq!(center.get(), None);

        center.update(100.0);
        assert_eq!(center.get(), Some(100.0));

        center.update(110.0);
        let result = center.get().unwrap();
        assert!((result - 103.33).abs() < 0.01); // (100*2 + 110)/3 ≈ 103.33
    }
}
