use super::{Attenuation, PowerLevel};
use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Command {
    RfPowerOn,
    RfPowerOff,
    StartAmpSweep {
        cw_freq_khz: f64,
        start_attenuation: Attenuation,
        start_power_level: PowerLevel,
        stop_attenuation: Attenuation,
        stop_power_level: PowerLevel,
        step_delay: Duration,
    },
    StartAmpSweepExp {
        cw_freq_khz: f64,
        start_power_dbm: f64,
        step_power_db: f64,
        stop_power_dbm: f64,
        step_delay: Duration,
    },
    StartCw {
        cw_freq_khz: f64,
        attenuation: Attenuation,
        power_level: PowerLevel,
    },
    StartCwExp {
        cw_freq_khz: f64,
        power_dbm: f64,
    },
    StartFreqSweep {
        start_freq_khz: f64,
        attenuation: Attenuation,
        power_level: PowerLevel,
        sweep_steps: u16,
        step_freq_khz: f64,
        step_delay: Duration,
    },
    StartFreqSweepExp {
        start_freq_khz: f64,
        power_dbm: f64,
        sweep_steps: u16,
        step_freq_khz: f64,
        step_delay: Duration,
    },
    StartTracking {
        start_freq_khz: f64,
        attenuation: Attenuation,
        power_level: PowerLevel,
        sweep_steps: u16,
        step_freq_khz: f64,
    },
    StartTrackingExp {
        start_freq_khz: f64,
        power_dbm: f64,
        sweep_steps: u16,
        step_freq_khz: f64,
    },
    TrackingStep(u16),
}

impl Command {
    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(self.clone())
    }
}

impl From<Command> for Vec<u8> {
    fn from(command: Command) -> Self {
        match command {
            Command::RfPowerOn => vec![b'#', 5, b'C', b'P', b'1'],
            Command::RfPowerOff => vec![b'#', 5, b'C', b'P', b'0'],
            Command::StartAmpSweep {
                cw_freq_khz,
                start_attenuation,
                start_power_level,
                stop_attenuation,
                stop_power_level,
                step_delay,
            } => {
                let mut command = vec![b'#', 30];
                command.extend(
                    format!(
                        "C3-A:{:07.0},{},{},{},{},{:05}",
                        cw_freq_khz,
                        u8::from(start_attenuation),
                        u8::from(start_power_level),
                        u8::from(stop_attenuation),
                        u8::from(stop_power_level),
                        step_delay.as_millis()
                    )
                    .bytes(),
                );
                command
            }
            Command::StartAmpSweepExp {
                cw_freq_khz,
                start_power_dbm,
                step_power_db,
                stop_power_dbm,
                step_delay,
            } => {
                let mut command = vec![b'#', 38];
                command.extend(
                    format!(
                        "C5-A:{:07.0},{:+05.1},{:+05.1},{:05.1},{:05}",
                        cw_freq_khz,
                        start_power_dbm,
                        step_power_db,
                        stop_power_dbm,
                        step_delay.as_millis()
                    )
                    .bytes(),
                );
                command
            }
            Command::StartCw {
                cw_freq_khz,
                attenuation,
                power_level,
            } => {
                let mut command = vec![b'#', 18];
                command.extend(
                    format!(
                        "C3-F:{:07.0},{},{}",
                        cw_freq_khz,
                        u8::from(attenuation),
                        u8::from(power_level)
                    )
                    .bytes(),
                );
                command
            }
            Command::StartCwExp {
                cw_freq_khz,
                power_dbm,
            } => {
                let mut command = vec![b'#', 20];
                command.extend(format!("C5-F:{:07.0},{:+05.1}", cw_freq_khz, power_dbm).bytes());
                command
            }
            Command::StartFreqSweep {
                start_freq_khz,
                attenuation,
                power_level,
                sweep_steps,
                step_freq_khz,
                step_delay,
            } => {
                let mut command = vec![b'#', 37];
                command.extend(
                    format!(
                        "C3-F:{:07.0},{},{},{:04},{:07.0},{:05}",
                        start_freq_khz,
                        u8::from(attenuation),
                        u8::from(power_level),
                        sweep_steps,
                        step_freq_khz,
                        step_delay.as_millis()
                    )
                    .bytes(),
                );
                command
            }
            Command::StartFreqSweepExp {
                start_freq_khz,
                power_dbm,
                sweep_steps,
                step_freq_khz,
                step_delay,
            } => {
                let mut command = vec![b'#', 39];
                command.extend(
                    format!(
                        "C5-F:{:07.0},{:+05.1},{:04},{:07.0},{:05}",
                        start_freq_khz,
                        power_dbm,
                        sweep_steps,
                        step_freq_khz,
                        step_delay.as_millis()
                    )
                    .bytes(),
                );
                command
            }
            Command::StartTracking {
                start_freq_khz,
                attenuation,
                power_level,
                sweep_steps,
                step_freq_khz,
            } => {
                let mut command = vec![b'#', 31];
                command.extend(
                    format!(
                        "C3-T:{:07.0},{},{},{:04},{:07.0}",
                        start_freq_khz,
                        u8::from(attenuation),
                        u8::from(power_level),
                        sweep_steps,
                        step_freq_khz
                    )
                    .bytes(),
                );
                command
            }
            Command::StartTrackingExp {
                start_freq_khz,
                power_dbm,
                sweep_steps,
                step_freq_khz,
            } => {
                let mut command = vec![b'#', 33];
                command.extend(
                    format!(
                        "C5-T:{:07.0},{:+05.1},{:04},{:07.0}",
                        start_freq_khz, power_dbm, sweep_steps, step_freq_khz
                    )
                    .bytes(),
                );
                command
            }
            Command::TrackingStep(steps) => {
                let steps_bytes = steps.to_be_bytes();
                vec![b'#', 5, b'k', steps_bytes[0], steps_bytes[1]]
            }
        }
    }
}
