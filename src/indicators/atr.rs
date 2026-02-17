/// Calculate True Range for a single bar
pub fn true_range(high: f64, low: f64, prev_close: f64) -> f64 {
    let tr1 = high - low;
    let tr2 = (high - prev_close).abs();
    let tr3 = (low - prev_close).abs();
    tr1.max(tr2).max(tr3)
}

/// ATR Calculator using RMA (Wilder's smoothing) to match PineScript
pub struct AtrCalculator {
    period: usize,
    atr: Option<f64>,
    tr_sum: f64,
    count: usize,
}

impl AtrCalculator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            atr: None,
            tr_sum: 0.0,
            count: 0,
        }
    }

    pub fn next(&mut self, high: f64, low: f64, prev_close: f64) -> f64 {
        let tr = true_range(high, low, prev_close);

        match self.atr {
            None => {
                // Build up the initial SMA
                self.tr_sum += tr;
                self.count += 1;

                if self.count >= self.period {
                    // Initialize with SMA
                    self.atr = Some(self.tr_sum / self.period as f64);
                }
                self.atr.unwrap_or(tr)
            }
            Some(prev_atr) => {
                // Use RMA (Wilder's smoothing): (prev_atr * (period - 1) + tr) / period
                let new_atr = (prev_atr * (self.period - 1) as f64 + tr) / self.period as f64;
                self.atr = Some(new_atr);
                new_atr
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_range() {
        // Test basic true range calculation
        let high = 110.0;
        let low = 105.0;
        let prev_close = 108.0;

        let tr = true_range(high, low, prev_close);
        assert_eq!(tr, 5.0); // max(110-105, |110-108|, |105-108|) = max(5, 2, 3) = 5
    }

    #[test]
    fn test_atr_calculator() {
        let mut atr = AtrCalculator::new(3);

        // First bar
        let result1 = atr.next(110.0, 105.0, 108.0);
        assert!(result1 > 0.0);

        // Second bar
        let result2 = atr.next(112.0, 108.0, 110.0);
        assert!(result2 > 0.0);

        // Third bar - should have ATR initialized
        let result3 = atr.next(115.0, 110.0, 112.0);
        assert!(result3 > 0.0);
        assert!(atr.atr.is_some());
    }
}
