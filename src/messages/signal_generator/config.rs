use rfe_message::RfeMessage;

#[derive(Debug, Copy, Clone, RfeMessage)]
#[prefix = "#C3-*:"]
pub struct Config {
    start_freq_khz: f64,
    cw_freq_khz: f64,
    // total_steps: usize,
    freq_step_khz: f64,
}

impl Config {
    pub fn start_freq_khz(&self) -> f64 {
        self.start_freq_khz
    }

    pub fn cw_freq_khz(&self) -> f64 {
        self.cw_freq_khz
    }

    pub fn freq_step_khz(&self) -> f64 {
        self.freq_step_khz
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn parse_config() {
        let bytes = b"#C3-*:5249000,0196428,2309412";
        let config = Config::try_from(bytes.as_ref()).unwrap();
        assert_eq!(config.start_freq_khz(), 5_249_000f64);
        assert_eq!(config.cw_freq_khz(), 196_428f64);
    }
}
