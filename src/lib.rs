mod indicators;

use exchange_outpost_abi::{Candle, FunctionArgs};
use extism_pdk::{FnResult, Json, ToBytes, encoding, info, plugin_fn};
use serde::Serialize;

use indicators::atr::AtrCalculator;
use indicators::pivot::{PivotCenterLine, pivot_high, pivot_low};
use indicators::supertrend::{SuperTrendSignal, SuperTrendState, Trend};

#[derive(Serialize)]
struct SignalData {
    index: usize,
    timestamp: i64,
    price: f64,
    signal_type: SuperTrendSignal,
    trend: Trend,
    signal_line: f64,
}

#[derive(Serialize, ToBytes)]
#[encoding(Json)]
pub struct Output {
    signals: Vec<SignalData>,
    final_trend: Option<Trend>,
}

#[plugin_fn]
pub fn run(call_args: FunctionArgs) -> FnResult<Output> {
    let ticker = call_args.get_ticker("symbol_data")?;
    let pivot_point_period: usize = call_args.get_call_argument("prd")?;
    let atr_factor: f64 = call_args.get_call_argument("factor")?;
    let atr_period: usize = call_args.get_call_argument("atr_prd")?;
    let candles: &Vec<Candle<f64>> = ticker.get_candles();

    let mut atr_calculator = AtrCalculator::new(atr_period);
    let mut center_line = PivotCenterLine::new();
    let mut supertrend_state: Option<SuperTrendState> = None;
    let mut signals = Vec::with_capacity(candles.len() / 20); // Estimate: At least ~5% of candles will be signals
    let mut last_pivot_high_idx: Option<usize> = None;
    let mut last_pivot_low_idx: Option<usize> = None;

    // Collect price arrays for pivot detection
    let mut highs = Vec::with_capacity(candles.len());
    let mut lows = Vec::with_capacity(candles.len());
    let mut closes = Vec::with_capacity(candles.len());
    let mut atrs = Vec::with_capacity(candles.len());

    // First pass: calculate ATR for all bars using RMA (matching PineScript)
    for (idx, candle) in candles.iter().enumerate() {
        highs.push(candle.high);
        lows.push(candle.low);
        closes.push(candle.close);

        let prev_close = if idx > 0 {
            candles[idx - 1].close
        } else {
            candle.close
        };

        let atr_value = atr_calculator.next(candle.high, candle.low, prev_close);
        atrs.push(atr_value);
    }

    // Second pass: detect pivots and calculate SuperTrend
    for i in 0..candles.len() {
        // Check for pivot high - only detect each pivot once
        // pivothigh(prd, prd) in PineScript checks if bar at i-prd is a pivot
        if i >= 2 * pivot_point_period {
            let pivot_idx = i - pivot_point_period;

            if last_pivot_high_idx.map_or(true, |idx| pivot_idx > idx) {
                if let Some(ph) = pivot_high(&highs[..=i], pivot_point_period, pivot_point_period) {
                    center_line.update(ph);
                    last_pivot_high_idx = Some(pivot_idx);
                }
            }

            if last_pivot_low_idx.map_or(true, |idx| pivot_idx > idx) {
                if let Some(pl) = pivot_low(&lows[..=i], pivot_point_period, pivot_point_period) {
                    center_line.update(pl);
                    last_pivot_low_idx = Some(pivot_idx);
                }
            }
        }

        // Calculate SuperTrend if we have a center line and ATR
        if let Some(center) = center_line.get() {
            let atr = atrs[i];
            if atr > 0.0 {
                let basic_upper = center + atr_factor * atr;
                let basic_lower = center - atr_factor * atr;

                match &mut supertrend_state {
                    None => {
                        // Initialize SuperTrend state (trend defaults to Up in PineScript)
                        supertrend_state = Some(SuperTrendState::new(basic_upper, basic_lower));
                    }
                    Some(state) => {
                        let old_trend = state.trend;
                        let prev_close = if i > 0 { closes[i - 1] } else { closes[i] };
                        state.update(basic_upper, basic_lower, closes[i], prev_close);

                        // Detect trend change (signal)
                        if old_trend != state.trend {
                            let signal_type = match state.trend {
                                Trend::Up => SuperTrendSignal::Long,
                                Trend::Down => SuperTrendSignal::Short,
                            };
                            info!("Signal detected at index {}: {}", i, signal_type);
                            signals.push(SignalData {
                                index: i,
                                timestamp: candles[i].timestamp,
                                price: closes[i],
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

    let final_trend = supertrend_state.as_ref().map(|s| s.trend);
    info!("Final trend: {:?}", final_trend);
    Ok(Output {
        signals,
        final_trend: final_trend,
    })
}
