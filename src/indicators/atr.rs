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
