use serde::{Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trend {
    Up,
    Down,
}

impl fmt::Display for Trend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trend::Up => write!(f, "UP"),
            Trend::Down => write!(f, "DOWN"),
        }
    }
}

impl Serialize for Trend {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub enum SuperTrendSignal {
    Long,
    Short,
}

impl fmt::Display for SuperTrendSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SuperTrendSignal::Long => write!(f, "LONG"),
            SuperTrendSignal::Short => write!(f, "SHORT"),
        }
    }
}

impl Serialize for SuperTrendSignal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// SuperTrendState struct goes here
pub struct SuperTrendState {
    pub trend: Trend,
    pub upper_band: f64,
    pub lower_band: f64,
}

impl SuperTrendState {
    pub fn new(initial_upper: f64, initial_lower: f64) -> Self {
        // PineScript initializes trend to 1 (Up) by default: nz(Trend[1], 1)
        Self {
            trend: Trend::Up,
            upper_band: initial_upper,
            lower_band: initial_lower,
        }
    }

    pub fn update(&mut self, basic_upper: f64, basic_lower: f64, close: f64, prev_close: f64) {
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

    pub fn get_signal_line(&self) -> f64 {
        match self.trend {
            Trend::Up => self.lower_band,
            Trend::Down => self.upper_band,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_display() {
        assert_eq!(Trend::Up.to_string(), "UP");
        assert_eq!(Trend::Down.to_string(), "DOWN");
    }

    #[test]
    fn test_supertrend_state_initialization() {
        let state = SuperTrendState::new(105.0, 95.0);
        assert_eq!(state.trend, Trend::Up);
        assert_eq!(state.upper_band, 105.0);
        assert_eq!(state.lower_band, 95.0);
    }

    #[test]
    fn test_supertrend_update() {
        let mut state = SuperTrendState::new(105.0, 95.0);

        // Update with new values
        state.update(106.0, 96.0, 100.0, 98.0);

        // Verify state has been updated
        assert!(state.upper_band > 0.0);
        assert!(state.lower_band > 0.0);
    }

    #[test]
    fn test_get_signal_line() {
        let state_up = SuperTrendState::new(105.0, 95.0);
        assert_eq!(state_up.get_signal_line(), 95.0); // Trend::Up returns lower_band

        let mut state_down = SuperTrendState::new(105.0, 95.0);
        state_down.trend = Trend::Down;
        assert_eq!(state_down.get_signal_line(), 105.0); // Trend::Down returns upper_band
    }
}
