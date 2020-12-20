use super::{CalcMode, DspMode, InputStage, WifiBand};

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Command {
    SetConfig {
        start_freq_khz: f64,
        stop_freq_khz: f64,
        min_amp_dbm: i16,
        max_amp_dbm: i16,
    },
    SwitchModuleMain,
    SwitchModuleExp,
    StartTracking {
        start_freq_khz: f64,
        step_freq_khz: f64,
    },
    StartWifiAnalyzer(WifiBand),
    StopWifiAnalyzer,
    SetCalcMode(CalcMode),
    TrackingStep(u16),
    SetDsp(DspMode),
    SetOffsetDB(i8),
    SetInputStage(InputStage),
    SetSweepPointsExt(u16),
    SetSweepPointsLarge(u16),
}

impl Command {
    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(self.clone())
    }
}

impl From<Command> for Vec<u8> {
    fn from(command: Command) -> Self {
        match command {
            Command::SetConfig {
                start_freq_khz,
                stop_freq_khz,
                min_amp_dbm,
                max_amp_dbm,
            } => {
                let mut command = vec![b'#', 32];
                command.extend(
                    format!(
                        "C2-F:{:07.0},{:07.0},{:04},{:04}",
                        start_freq_khz, stop_freq_khz, max_amp_dbm, min_amp_dbm
                    )
                    .bytes(),
                );
                command
            }
            Command::SwitchModuleMain => vec![b'#', 5, b'C', b'M', 0],
            Command::SwitchModuleExp => vec![b'#', 5, b'C', b'M', 1],
            Command::StartTracking {
                start_freq_khz,
                step_freq_khz,
            } => {
                let mut command = vec![b'#', 22];
                command
                    .extend(format!("C3-K:{:07.0},{:07.0}", start_freq_khz, step_freq_khz).bytes());
                command
            }
            Command::StartWifiAnalyzer(wifi_band) => vec![b'#', 5, b'C', b'W', u8::from(wifi_band)],
            Command::StopWifiAnalyzer => vec![b'#', 5, b'C', b'W', 0],
            Command::SetCalcMode(calc_mode) => vec![b'#', 5, b'C', b'+', u8::from(calc_mode)],
            Command::TrackingStep(steps) => {
                let steps_bytes = steps.to_be_bytes();
                vec![b'#', 5, b'k', steps_bytes[0], steps_bytes[1]]
            }
            Command::SetDsp(dsp_mode) => vec![b'#', 5, b'C', b'p', u8::from(dsp_mode)],
            Command::SetOffsetDB(offset_db) => vec![b'#', 5, b'C', b'O', offset_db as u8],
            Command::SetInputStage(input_stage) => vec![b'#', 4, b'a', u8::from(input_stage)],
            Command::SetSweepPointsExt(sweep_points) => {
                vec![b'#', 5, b'C', b'J', ((sweep_points / 16) - 1) as u8]
            }
            Command::SetSweepPointsLarge(sweep_points) => {
                let sweep_point_bytes = sweep_points.to_be_bytes();
                vec![
                    b'#',
                    6,
                    b'C',
                    b'j',
                    sweep_point_bytes[0],
                    sweep_point_bytes[1],
                ]
            }
        }
    }
}
