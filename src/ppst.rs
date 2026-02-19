use crate::{
    Trend,
    indicators::atr::AtrCalculator,
    indicators::pivot::{PivotCenterLine, pivot_high, pivot_low},
    indicators::supertrend::{SuperTrendSignal, SuperTrendState},
};
use exchange_outpost_abi::Candle;
use extism_pdk::info;
use serde::Serialize;

pub enum ComputationState {
    Initialized,
    Completed,
}

#[derive(Serialize)]
pub struct SignalData {
    index: usize,
    timestamp: i64,
    price: f64,
    signal_type: SuperTrendSignal,
    trend: Trend,
    signal_line: f64,
}

pub struct PPST {
    // Parameters
    pub pivot_point_period: usize,
    pub atr_factor: f64,
    pub atr_period: usize,
    // State
    pub computation_state: ComputationState,
    pub atr_calculator: AtrCalculator,
    pub center_line: PivotCenterLine,
    pub supertrend_state: Option<SuperTrendState>,
    pub signals: Vec<SignalData>,
    pub last_pivot_high_idx: Option<usize>,
    pub last_pivot_low_idx: Option<usize>,
    pub highs: Vec<f64>,
    pub lows: Vec<f64>,
    pub closes: Vec<f64>,
    pub atrs: Vec<f64>,
}

impl PPST {
    pub fn new(
        pivot_point_period: usize,
        atr_factor: f64,
        atr_period: usize,
        candles_count: usize,
    ) -> Self {
        PPST {
            pivot_point_period,
            atr_factor,
            atr_period,
            computation_state: ComputationState::Initialized,
            atr_calculator: AtrCalculator::new(atr_period),
            center_line: PivotCenterLine::new(),
            supertrend_state: None,
            signals: Vec::with_capacity(candles_count / 20), // Estimate: At least ~5% of candles will be signals
            last_pivot_high_idx: None,
            last_pivot_low_idx: None,
            highs: Vec::with_capacity(candles_count),
            lows: Vec::with_capacity(candles_count),
            closes: Vec::with_capacity(candles_count),
            atrs: Vec::with_capacity(candles_count),
        }
    }

    pub fn reset(&mut self) {
        self.computation_state = ComputationState::Initialized;
        self.atr_calculator = AtrCalculator::new(self.atr_period);
        self.center_line = PivotCenterLine::new();
        self.supertrend_state = None;
        self.signals.clear();
        self.last_pivot_high_idx = None;
        self.last_pivot_low_idx = None;
        self.highs.clear();
        self.lows.clear();
        self.closes.clear();
        self.atrs.clear();
    }

    pub fn calculate(&mut self, candles: &Vec<Candle<f64>>) {
        match self.computation_state {
            ComputationState::Completed => {
                self.reset();
            }
            ComputationState::Initialized => {
                // Continue with calculation
            }
        }

        // First pass: calculate ATR for all bars using RMA (matching PineScript)
        for (idx, candle) in candles.iter().enumerate() {
            self.highs.push(candle.high);
            self.lows.push(candle.low);
            self.closes.push(candle.close);

            let prev_close = if idx > 0 {
                candles[idx - 1].close
            } else {
                candle.close
            };

            let atr_value = self
                .atr_calculator
                .next(candle.high, candle.low, prev_close);
            self.atrs.push(atr_value);
        }

        // Second pass: detect pivots and calculate SuperTrend
        for i in 0..candles.len() {
            // Check for pivot high - only detect each pivot once
            // pivothigh(prd, prd) in PineScript checks if bar at i-prd is a pivot
            if i >= 2 * self.pivot_point_period {
                let pivot_idx = i - self.pivot_point_period;

                if self.last_pivot_high_idx.map_or(true, |idx| pivot_idx > idx) {
                    if let Some(ph) = pivot_high(
                        &self.highs[..=i],
                        self.pivot_point_period,
                        self.pivot_point_period,
                    ) {
                        self.center_line.update(ph);
                        self.last_pivot_high_idx = Some(pivot_idx);
                    }
                }

                if self.last_pivot_low_idx.map_or(true, |idx| pivot_idx > idx) {
                    if let Some(pl) = pivot_low(
                        &self.lows[..=i],
                        self.pivot_point_period,
                        self.pivot_point_period,
                    ) {
                        self.center_line.update(pl);
                        self.last_pivot_low_idx = Some(pivot_idx);
                    }
                }

                // Calculate SuperTrend if we have a center line and ATR
                if let Some(center) = self.center_line.get() {
                    let atr = self.atrs[i];
                    if atr > 0.0 {
                        let basic_upper = center + self.atr_factor * atr;
                        let basic_lower = center - self.atr_factor * atr;

                        match &mut self.supertrend_state {
                            None => {
                                // Initialize SuperTrend state (trend defaults to Up in PineScript)
                                self.supertrend_state =
                                    Some(SuperTrendState::new(basic_upper, basic_lower));
                            }
                            Some(state) => {
                                let old_trend = state.trend;
                                let prev_close = if i > 0 {
                                    self.closes[i - 1]
                                } else {
                                    self.closes[i]
                                };
                                state.update(basic_upper, basic_lower, self.closes[i], prev_close);

                                // Detect trend change (signal)
                                if old_trend != state.trend {
                                    let signal_type = match state.trend {
                                        Trend::Up => SuperTrendSignal::Long,
                                        Trend::Down => SuperTrendSignal::Short,
                                    };
                                    info!("Signal detected at index {}: {}", i, signal_type);
                                    self.signals.push(SignalData {
                                        index: i,
                                        timestamp: candles[i].timestamp,
                                        price: self.closes[i],
                                        signal_type: signal_type,
                                        trend: state.trend,
                                        signal_line: state.get_signal_line(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        self.computation_state = ComputationState::Completed;
    }
}
