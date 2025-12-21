mod exchange_outpost;
use crate::exchange_outpost::FinData;
use extism_pdk::{FnResult, Json, ToBytes, encoding, plugin_fn};
use serde::Serialize;

/// Calculate True Range for a single bar
fn true_range(high: f64, low: f64, prev_close: f64) -> f64 {
    let tr1 = high - low;
    let tr2 = (high - prev_close).abs();
    let tr3 = (low - prev_close).abs();
    tr1.max(tr2).max(tr3)
}

/// ATR Calculator using RMA (Wilder's smoothing) to match PineScript
struct AtrCalculator {
    period: usize,
    atr: Option<f64>,
    tr_sum: f64,
    count: usize,
}

impl AtrCalculator {
    fn new(period: usize) -> Self {
        Self {
            period,
            atr: None,
            tr_sum: 0.0,
            count: 0,
        }
    }

    fn next(&mut self, high: f64, low: f64, prev_close: f64) -> f64 {
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
struct PivotCenterLine {
    center: Option<f64>,
}

impl PivotCenterLine {
    fn new() -> Self {
        Self { center: None }
    }

    fn update(&mut self, pivot_price: f64) {
        self.center = Some(match self.center {
            None => pivot_price,
            Some(prev) => (prev * 2.0 + pivot_price) / 3.0,
        });
    }

    fn get(&self) -> Option<f64> {
        self.center
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Trend {
    Up,
    Down,
}

struct SuperTrendState {
    trend: Trend,
    upper_band: f64,
    lower_band: f64,
}

impl SuperTrendState {
    fn new(initial_upper: f64, initial_lower: f64) -> Self {
        // PineScript initializes trend to 1 (Up) by default: nz(Trend[1], 1)
        Self {
            trend: Trend::Up,
            upper_band: initial_upper,
            lower_band: initial_lower,
        }
    }

    fn update(&mut self, basic_upper: f64, basic_lower: f64, close: f64, prev_close: f64) {
        // Update bands with trailing logic (matching PineScript)
        // TUp := close[1] > TUp[1] ? max(Up, TUp[1]) : Up
        let new_lower = if prev_close > self.lower_band {
            basic_lower.max(self.lower_band)
        } else {
            basic_lower
        };

        // TDown := close[1] < TDown[1] ? min(Dn, TDown[1]) : Dn
        let new_upper = if prev_close < self.upper_band {
            basic_upper.min(self.upper_band)
        } else {
            basic_upper
        };

        // Determine trend using previous band values (matching PineScript)
        // Trend := close > TDown[1] ? 1: close < TUp[1]? -1: nz(Trend[1], 1)
        let new_trend = if close > self.upper_band {
            Trend::Up
        } else if close < self.lower_band {
            Trend::Down
        } else {
            self.trend
        };

        self.trend = new_trend;
        self.upper_band = new_upper;
        self.lower_band = new_lower;
    }

    fn get_signal_line(&self) -> f64 {
        match self.trend {
            Trend::Up => self.lower_band,
            Trend::Down => self.upper_band,
        }
    }
}

#[derive(Serialize)]
struct SignalData {
    index: usize,
    timestamp: i64,
    price: f64,
    signal_type: String,
    trend: String,
    signal_line: f64,
}

#[derive(Serialize, ToBytes)]
#[encoding(Json)]
pub struct Output {
    // signals: Vec<SignalData>,
    final_trend: String,
}

#[plugin_fn]
pub fn run(fin_data: FinData) -> FnResult<Output> {
    // let ticker = fin_data.get_ticker("symbol_data")?;
    // let pivot_point_period: usize = fin_data.get_call_argument("prd")?;
    // let atr_factor: f64 = fin_data.get_call_argument("factor")?;
    // let atr_period: usize = fin_data.get_call_argument("atr_prd")?;
    // let candles: &Vec<exchange_outpost::Candle<f64>> = ticker.get_candles();

    // let mut atr_calculator = AtrCalculator::new(atr_period);
    // let mut center_line = PivotCenterLine::new();
    // let mut supertrend_state: Option<SuperTrendState> = None;
    // let mut signals = Vec::new();
    // let mut last_pivot_high_idx: Option<usize> = None;
    // let mut last_pivot_low_idx: Option<usize> = None;

    // // Collect price arrays for pivot detection
    // let mut highs = Vec::new();
    // let mut lows = Vec::new();
    // let mut closes = Vec::new();
    // let mut atrs = Vec::new();

    // // First pass: calculate ATR for all bars using RMA (matching PineScript)
    // for (idx, candle) in candles.iter().enumerate() {
    //     highs.push(candle.high);
    //     lows.push(candle.low);
    //     closes.push(candle.close);

    //     let prev_close = if idx > 0 {
    //         candles[idx - 1].close
    //     } else {
    //         candle.close
    //     };

    //     let atr_value = atr_calculator.next(candle.high, candle.low, prev_close);
    //     atrs.push(atr_value);
    // }

    // // Second pass: detect pivots and calculate SuperTrend
    // for i in 0..candles.len() {
    //     // Check for pivot high - only detect each pivot once
    //     // pivothigh(prd, prd) in PineScript checks if bar at i-prd is a pivot
    //     if i >= 2 * pivot_point_period {
    //         let pivot_idx = i - pivot_point_period;

    //         if last_pivot_high_idx.map_or(true, |idx| pivot_idx > idx) {
    //             if let Some(ph) = pivot_high(&highs[..=i], pivot_point_period, pivot_point_period) {
    //                 center_line.update(ph);
    //                 last_pivot_high_idx = Some(pivot_idx);
    //             }
    //         }

    //         if last_pivot_low_idx.map_or(true, |idx| pivot_idx > idx) {
    //             if let Some(pl) = pivot_low(&lows[..=i], pivot_point_period, pivot_point_period) {
    //                 center_line.update(pl);
    //                 last_pivot_low_idx = Some(pivot_idx);
    //             }
    //         }
    //     }

    //     // Calculate SuperTrend if we have a center line and ATR
    //     if let Some(center) = center_line.get() {
    //         let atr = atrs[i];
    //         if atr > 0.0 {
    //             let basic_upper = center + atr_factor * atr;
    //             let basic_lower = center - atr_factor * atr;

    //             match &mut supertrend_state {
    //                 None => {
    //                     // Initialize SuperTrend state (trend defaults to Up in PineScript)
    //                     supertrend_state = Some(SuperTrendState::new(basic_upper, basic_lower));
    //                 }
    //                 Some(state) => {
    //                     let old_trend = state.trend;
    //                     let prev_close = if i > 0 { closes[i - 1] } else { closes[i] };
    //                     state.update(basic_upper, basic_lower, closes[i], prev_close);

    //                     // Detect trend change (signal)
    //                     if old_trend != state.trend {
    //                         let signal_type = match state.trend {
    //                             Trend::Up => "LONG",
    //                             Trend::Down => "SHORT",
    //                         };

    //                         signals.push(SignalData {
    //                             index: i,
    //                             timestamp: candles[i].timestamp,
    //                             price: closes[i],
    //                             signal_type: signal_type.to_string(),
    //                             trend: format!("{:?}", state.trend),
    //                             signal_line: state.get_signal_line(),
    //                         });
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    // let final_trend = supertrend_state
    //     .as_ref()
    //     .map(|s| format!("{:?}", s.trend))
    //     .unwrap_or_else(|| "Unknown".to_string());

    Ok(Output {
        // signals,
        final_trend: fin_data.get_ticker("pegged_data")?.symbol.clone(),
    })
}
