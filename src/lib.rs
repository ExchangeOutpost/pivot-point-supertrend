mod indicators;
mod ppst;
use exchange_outpost_abi::{Candle, FunctionArgs};
use extism_pdk::{FnResult, Json, ToBytes, encoding, info, plugin_fn};
use indicators::supertrend::Trend;
use ppst::{PPST, SignalData};
use serde::Serialize;

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

    let mut ppst = PPST::new(pivot_point_period, atr_factor, atr_period, candles.len());
    ppst.calculate(candles);

    let final_trend = ppst.supertrend_state.as_ref().map(|s| s.trend);
    info!("Final trend: {:?}", final_trend);
    Ok(Output {
        signals: ppst.signals,
        final_trend: final_trend,
    })
}
