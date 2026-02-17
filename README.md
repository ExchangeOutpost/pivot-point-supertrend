# Pivot Point Supertrend

A technical indicator that combines pivot points with the SuperTrend to generate trading signals. Detects trend changes based on pivot-adjusted support and resistance levels.

## Input Parameters

| Parameter | Type | Description | Default | Range |
|-----------|------|-------------|---------|-------|
| `prd` | integer | Pivot Point Period - lookback period for detecting pivot highs/lows | 2 | 1-50 |
| `factor` | number | ATR Factor - multiplier for ATR to determine band width | 3 | ≥1 |
| `atr_prd` | integer | ATR Period - period for Average True Range calculation | 10 | ≥1 |

**Required Financial Data**: `symbol_data` (OHLC candles)

## Output Structure

```json
{
  "signals": [
    {
      "index": 45,
      "timestamp": 1708176000,
      "price": 150.25,
      "signal_type": "Long",
      "trend": "Up",
      "signal_line": 148.30
    }
  ],
  "final_trend": "Up"
}
```

### Output Fields

- **signals**: Array of detected trend change signals
  - `index`: Candle index where signal occurred
  - `timestamp`: Unix timestamp of the signal
  - `price`: Close price at signal
  - `signal_type`: `"Long"` (buy) or `"Short"` (sell)
  - `trend`: Current trend after signal (`"Up"` or `"Down"`)
  - `signal_line`: SuperTrend line value at signal
- **final_trend**: Last trend state (`"Up"`, `"Down"`, or `null` if not calculated)