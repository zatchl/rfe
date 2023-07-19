use std::{
    fmt::Debug,
    io,
    ops::{Deref, RangeInclusive},
    sync::{Condvar, Mutex, MutexGuard, WaitTimeoutResult},
    time::Duration,
};

use num_enum::IntoPrimitive;
use tracing::{error, info, trace, warn};

use super::{CalcMode, Command, Config, DspMode, InputStage, Sweep, TrackingStatus};
use crate::common::{ConnectionResult, Error, Frequency, Result};
use crate::rf_explorer::{
    Callback, RadioModule, RfExplorer, RfExplorerMessageContainer, ScreenData, SerialNumber,
    SetupInfo,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive)]
#[repr(u8)]
pub enum WifiBand {
    TwoPointFourGhz = 1,
    FiveGhz,
}

#[derive(Debug)]
pub struct SpectrumAnalyzer {
    rfe: RfExplorer<MessageContainer>,
}

impl SpectrumAnalyzer {
    const MIN_MAX_AMP_RANGE_DBM: RangeInclusive<i16> = -120..=35;
    const MIN_SWEEP_POINTS: u16 = 112;
    const NEXT_SWEEP_TIMEOUT: Duration = Duration::from_secs(2);

    pub fn connect() -> Option<Self> {
        Some(Self {
            rfe: RfExplorer::connect()?,
        })
    }

    pub fn connect_with_name_and_baud_rate(name: &str, baud_rate: u32) -> ConnectionResult<Self> {
        Ok(Self {
            rfe: RfExplorer::connect_with_name_and_baud_rate(name, baud_rate)?,
        })
    }

    pub fn connect_all() -> Vec<Self> {
        RfExplorer::connect_all()
            .into_iter()
            .map(|rfe| Self { rfe })
            .collect()
    }

    pub fn serial_number(&self) -> io::Result<crate::SerialNumber> {
        if let Some(ref serial_number) = *self.message_container().serial_number.0.lock().unwrap() {
            return Ok(serial_number.clone());
        }

        self.send_command(crate::rf_explorer::Command::RequestSerialNumber)?;

        let (lock, cvar) = &self.message_container().serial_number;
        tracing::trace!("Waiting to receive SerialNumber from RF Explorer");
        let _ = cvar
            .wait_timeout_while(
                lock.lock().unwrap(),
                std::time::Duration::from_secs(2),
                |serial_number| serial_number.is_none(),
            )
            .unwrap();

        if let Some(ref serial_number) = *self.message_container().serial_number.0.lock().unwrap() {
            Ok(serial_number.clone())
        } else {
            Err(io::ErrorKind::TimedOut.into())
        }
    }

    /// Returns the RF Explorer's current `Config`.
    pub fn config(&self) -> Config {
        self.message_container()
            .config
            .0
            .lock()
            .unwrap()
            .unwrap_or_default()
    }

    /// Returns the most recent `Sweep` measured by the RF Explorer.
    pub fn sweep(&self) -> Option<Sweep> {
        self.message_container().sweep.0.lock().unwrap().clone()
    }

    /// Waits for the RF Explorer to measure its next `Sweep`.
    pub fn wait_for_next_sweep(&self) -> Result<Sweep> {
        self.wait_for_next_sweep_with_timeout(Self::NEXT_SWEEP_TIMEOUT)
    }

    /// Waits for the RF Explorer to measure its next `Sweep` or for the timeout duration to elapse.
    pub fn wait_for_next_sweep_with_timeout(&self, timeout: Duration) -> Result<Sweep> {
        let previous_sweep = self.sweep();

        let (sweep, cond_var) = &self.message_container().sweep;
        let (sweep, wait_result) = cond_var
            .wait_timeout_while(sweep.lock().unwrap(), timeout, |sweep| {
                *sweep == previous_sweep || sweep.is_none()
            })
            .unwrap();

        match &*sweep {
            Some(sweep) if !wait_result.timed_out() => Ok(sweep.clone()),
            _ => Err(Error::TimedOut(timeout)),
        }
    }

    /// Returns the most recent `ScreenData` captured by the RF Explorer.
    pub fn screen_data(&self) -> Option<ScreenData> {
        self.message_container()
            .screen_data
            .0
            .lock()
            .unwrap()
            .clone()
    }

    /// Waits for the RF Explorer to capture its next `ScreenData`.
    pub fn wait_for_next_screen_data(&self) -> Result<ScreenData> {
        self.wait_for_next_screen_data_with_timeout(
            RfExplorer::<MessageContainer>::NEXT_SCREEN_DATA_TIMEOUT,
        )
    }

    /// Waits for the RF Explorer to capture its next `ScreenData` or for the timeout duration to elapse.
    pub fn wait_for_next_screen_data_with_timeout(&self, timeout: Duration) -> Result<ScreenData> {
        let previous_screen_data = self.screen_data();

        let (screen_data, condvar) = &self.message_container().screen_data;
        let (screen_data, wait_result) = condvar
            .wait_timeout_while(screen_data.lock().unwrap(), timeout, |screen_data| {
                *screen_data == previous_screen_data || screen_data.is_none()
            })
            .unwrap();

        match &*screen_data {
            Some(screen_data) if !wait_result.timed_out() => Ok(screen_data.clone()),
            _ => Err(Error::TimedOut(timeout)),
        }
    }

    /// Returns the RF Explorer's DSP mode.
    pub fn dsp_mode(&self) -> Option<DspMode> {
        *self.message_container().dsp_mode.0.lock().unwrap()
    }

    /// Returns the status of tracking mode (enabled or disabled).
    pub fn tracking_status(&self) -> Option<TrackingStatus> {
        *self.message_container().tracking_status.0.lock().unwrap()
    }

    pub fn input_stage(&self) -> Option<InputStage> {
        *self.message_container().input_stage.0.lock().unwrap()
    }

    /// Returns the main radio module.
    pub fn main_radio_module(&self) -> RadioModule {
        self.message_container()
            .setup_info
            .0
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .main_radio_module
    }

    /// Returns the expansion radio module (if one exists).
    pub fn expansion_radio_module(&self) -> Option<RadioModule> {
        self.message_container()
            .setup_info
            .0
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .expansion_radio_module
    }

    /// Returns the active radio module.
    pub fn active_radio_module(&self) -> RadioModule {
        if self.config().is_expansion_radio_module_active {
            self.expansion_radio_module().unwrap()
        } else {
            self.main_radio_module()
        }
    }

    /// Returns the inactive radio module (if one exists).
    pub fn inactive_radio_module(&self) -> Option<RadioModule> {
        let expansion_radio_module = self.expansion_radio_module();
        if expansion_radio_module.is_some() {
            if self.config().is_expansion_radio_module_active {
                Some(self.main_radio_module())
            } else {
                expansion_radio_module
            }
        } else {
            None
        }
    }

    /// Starts the spectrum analyzer's Wi-Fi analyzer.
    #[tracing::instrument]
    pub fn start_wifi_analyzer(&self, wifi_band: WifiBand) -> io::Result<()> {
        self.send_command(Command::StartWifiAnalyzer(wifi_band))
    }

    /// Stops the spectrum analyzer's Wi-Fi analyzer.
    #[tracing::instrument(skip(self))]
    pub fn stop_wifi_analyzer(&self) -> io::Result<()> {
        self.send_command(Command::StopWifiAnalyzer)
    }

    /// Requests the spectrum analyzer enter tracking mode.
    #[tracing::instrument(skip(self))]
    pub fn request_tracking(&self, start_hz: u64, step_hz: u64) -> Result<TrackingStatus> {
        // Set the tracking status to None so we can tell whether or not we've received a new
        // tracking status message by checking for Some
        *self.message_container().tracking_status.0.lock().unwrap() = None;

        // Send the command to enter tracking mode
        self.send_command(Command::StartTracking {
            start: Frequency::from_hz(start_hz),
            step: Frequency::from_hz(step_hz),
        })?;

        // Wait to see if we receive a tracking status message in response
        let (lock, condvar) = &self.message_container().tracking_status;
        let (tracking_status, wait_result) = condvar
            .wait_timeout_while(
                lock.lock().unwrap(),
                RfExplorer::<MessageContainer>::COMMAND_RESPONSE_TIMEOUT,
                |tracking_status| tracking_status.is_some(),
            )
            .unwrap();

        if !wait_result.timed_out() {
            Ok(tracking_status.unwrap_or_default())
        } else {
            Err(Error::TimedOut(
                RfExplorer::<MessageContainer>::COMMAND_RESPONSE_TIMEOUT,
            ))
        }
    }

    /// Steps over the tracking step frequency and makes a measurement.
    #[tracing::instrument(skip(self))]
    pub fn tracking_step(&self, step: u16) -> io::Result<()> {
        self.send_command(Command::TrackingStep(step))
    }

    /// Activates the RF Explorer's main radio module.
    pub fn activate_main_radio_module(&self) -> Result<()> {
        if self.active_radio_module().is_main() {
            return Err(Error::InvalidOperation(
                "Main radio module is already active.".to_string(),
            ));
        }

        self.send_command(Command::SwitchModuleMain)?;

        // Wait until config shows that the main radio module is active
        let _ = self.wait_for_config_while(|config| {
            config
                .filter(|config| !config.is_expansion_radio_module_active)
                .is_none()
        });

        if self.active_radio_module().is_main() {
            Ok(())
        } else {
            Err(Error::TimedOut(
                RfExplorer::<MessageContainer>::COMMAND_RESPONSE_TIMEOUT,
            ))
        }
    }

    /// Activates the RF Explorer's expansion radio module (if one exists).
    pub fn activate_expansion_radio_module(&self) -> Result<()> {
        if self.expansion_radio_module().is_none() {
            return Err(Error::InvalidOperation(
                "This RF Explorer does not contain an expansion radio module.".to_string(),
            ));
        }

        if self.active_radio_module().is_expansion() {
            return Err(Error::InvalidOperation(
                "Expansion radio module is already active.".to_string(),
            ));
        }

        self.send_command(Command::SwitchModuleExp)?;

        // Wait until config shows that the expansion radio module is active
        let _ = self.wait_for_config_while(|config| {
            config
                .filter(|config| config.is_expansion_radio_module_active)
                .is_none()
        });

        if self.active_radio_module().is_expansion() {
            Ok(())
        } else {
            Err(Error::TimedOut(
                RfExplorer::<MessageContainer>::COMMAND_RESPONSE_TIMEOUT,
            ))
        }
    }

    /// Sets the start and stop frequency of sweeps measured by the spectrum analyzer.
    pub fn set_start_stop(
        &self,
        start: impl Into<Frequency>,
        stop: impl Into<Frequency>,
    ) -> Result<()> {
        let config = self.config();
        self.set_config(
            start.into(),
            stop.into(),
            config.min_amp_dbm,
            config.max_amp_dbm,
        )
    }

    /// Sets the start frequency, stop frequency, and number of points of sweeps measured by the spectrum analyzer.
    pub fn set_start_stop_sweep_points(
        &self,
        start: impl Into<Frequency>,
        stop: impl Into<Frequency>,
        sweep_points: u16,
    ) -> Result<()> {
        let (start, stop) = (start.into(), stop.into());
        let config = self.config();
        self.set_sweep_points(sweep_points)?;
        self.set_config(start, stop, config.min_amp_dbm, config.max_amp_dbm)
    }

    /// Sets the center frequency and span of sweeps measured by the spectrum analyzer.
    pub fn set_center_span(
        &self,
        center: impl Into<Frequency>,
        span: impl Into<Frequency>,
    ) -> Result<()> {
        let (center, span) = (center.into(), span.into());
        self.set_start_stop(center - span / 2, center + span / 2)
    }

    /// Sets the center frequency, span, and number of points of sweeps measured by the spectrum analyzer.
    pub fn set_center_span_sweep_points(
        &self,
        center: impl Into<Frequency>,
        span: impl Into<Frequency>,
        sweep_points: u16,
    ) -> Result<()> {
        let (center, span) = (center.into(), span.into());
        self.set_start_stop_sweep_points(center - span / 2, center + span / 2, sweep_points)
    }

    /// Sets the minimum and maximum amplitudes displayed on the RF Explorer's screen.
    #[tracing::instrument(skip(self))]
    pub fn set_min_max_amps(&self, min_amp_dbm: i16, max_amp_dbm: i16) -> Result<()> {
        let config = self.config();
        self.set_config(config.start, config.stop, min_amp_dbm, max_amp_dbm)
    }

    /// Sets the spectrum analyzer's configuration.
    #[tracing::instrument(skip(self), ret, err)]
    fn set_config(
        &self,
        start: Frequency,
        stop: Frequency,
        min_amp_dbm: i16,
        max_amp_dbm: i16,
    ) -> Result<()> {
        self.validate_start_stop(start, stop)?;
        self.validate_min_max_amps(min_amp_dbm, max_amp_dbm)?;

        self.send_command(Command::SetConfig {
            start,
            stop,
            min_amp_dbm,
            max_amp_dbm,
        })?;

        // Check if the current config already contains the requested values
        if self
            .config()
            .contains_start_stop_amp_range(start, stop, min_amp_dbm, max_amp_dbm)
        {
            return Ok(());
        }

        // Wait until the current config contains the requested values
        trace!("Waiting to receive updated 'Config'");
        let (_, wait_result) = self.wait_for_config_while(|config| {
            let Some(config) = config else {
                    return true;
                };

            !config.contains_start_stop_amp_range(start, stop, min_amp_dbm, max_amp_dbm)
        });

        if !wait_result.timed_out() {
            Ok(())
        } else {
            Err(Error::TimedOut(
                RfExplorer::<MessageContainer>::COMMAND_RESPONSE_TIMEOUT,
            ))
        }
    }

    /// Sets the callback that is called when the spectrum analyzer receives a `Sweep`.
    pub fn set_sweep_callback(&self, cb: impl FnMut(Sweep) + Send + 'static) {
        *self.message_container().sweep_callback.lock().unwrap() = Some(Box::new(cb));
    }

    /// Sets the callback that is called when the spectrum analyzer receives a `Config`.
    pub fn set_config_callback(&self, cb: impl FnMut(Config) + Send + 'static) {
        *self.message_container().config_callback.lock().unwrap() = Some(Box::new(cb));
    }

    /// Sets the number of points in each sweep measured by the spectrum analyzer.
    #[tracing::instrument(skip(self))]
    pub fn set_sweep_points(&self, sweep_points: u16) -> Result<()> {
        // Only 'Plus' models can set the number of points in a sweep
        if !self.active_radio_module().model().is_plus_model() {
            return Err(Error::InvalidOperation(
                "Only RF Explorer 'Plus' models support setting the number of sweep points"
                    .to_string(),
            ));
        }

        if sweep_points <= 4096 {
            self.send_command(Command::SetSweepPointsExt(sweep_points))?;
        } else {
            self.send_command(Command::SetSweepPointsLarge(sweep_points))?;
        }

        // The requested number of sweep points gets rounded down to a number that's a multiple of 16
        let expected_sweep_points = if sweep_points < 112 {
            Self::MIN_SWEEP_POINTS
        } else {
            (sweep_points / 16) * 16
        };

        // Check if the current config already contains the requested sweep points
        if self.config().sweep_points == expected_sweep_points {
            return Ok(());
        }

        // Wait until the current config contains the requested sweep points
        info!("Waiting to receive updated config");
        let (_, wait_result) = self.wait_for_config_while(|config| {
            config
                .filter(|config| config.sweep_points == expected_sweep_points)
                .is_none()
        });

        if !wait_result.timed_out() {
            Ok(())
        } else {
            warn!("Failed to receive updated config");
            Err(Error::TimedOut(
                RfExplorer::<MessageContainer>::COMMAND_RESPONSE_TIMEOUT,
            ))
        }
    }

    /// Sets the spectrum analyzer's calculator mode.
    #[tracing::instrument(skip(self))]
    pub fn set_calc_mode(&self, calc_mode: CalcMode) -> io::Result<()> {
        self.send_command(Command::SetCalcMode(calc_mode))
    }

    /// Sets the spectrum analyzer's input stage.
    #[tracing::instrument(skip(self))]
    pub fn set_input_stage(&self, input_stage: InputStage) -> io::Result<()> {
        self.send_command(Command::SetInputStage(input_stage))
    }

    /// Adds or subtracts an offset to the amplitudes in each sweep.
    #[tracing::instrument(skip(self))]
    pub fn set_offset_db(&self, offset_db: i8) -> io::Result<()> {
        self.send_command(Command::SetOffsetDB(offset_db))
    }

    /// Sets the spectrum analyzer's DSP mode.
    #[tracing::instrument(skip(self))]
    pub fn set_dsp_mode(&self, dsp_mode: DspMode) -> Result<()> {
        // Check to see if the DspMode is already set to the desired value
        if *self.message_container().dsp_mode.0.lock().unwrap() == Some(dsp_mode) {
            return Ok(());
        }

        // Send the command to set the DSP mode
        self.send_command(Command::SetDsp(dsp_mode))?;

        // Wait to see if we receive a DSP mode message in response
        let (lock, condvar) = &self.message_container().dsp_mode;
        let (_, wait_result) = condvar
            .wait_timeout_while(
                lock.lock().unwrap(),
                RfExplorer::<MessageContainer>::COMMAND_RESPONSE_TIMEOUT,
                |new_dsp_mode| *new_dsp_mode != Some(dsp_mode),
            )
            .unwrap();

        if !wait_result.timed_out() {
            Ok(())
        } else {
            Err(Error::TimedOut(
                RfExplorer::<MessageContainer>::COMMAND_RESPONSE_TIMEOUT,
            ))
        }
    }

    fn wait_for_config_while(
        &self,
        condition: impl FnMut(&mut Option<Config>) -> bool,
    ) -> (MutexGuard<Option<Config>>, WaitTimeoutResult) {
        let (lock, condvar) = &self.message_container().config;
        condvar
            .wait_timeout_while(
                lock.lock().unwrap(),
                RfExplorer::<MessageContainer>::COMMAND_RESPONSE_TIMEOUT,
                condition,
            )
            .unwrap()
    }

    #[tracing::instrument(skip(self), ret, err)]
    fn validate_start_stop(&self, start: Frequency, stop: Frequency) -> Result<()> {
        if start >= stop {
            return Err(Error::InvalidInput(
                "The start frequency must be less than the stop frequency".to_string(),
            ));
        }

        let active_model = self.active_radio_module().model();

        let min_max_freq = active_model.min_freq()..=active_model.max_freq();
        if !min_max_freq.contains(&start) {
            return Err(Error::InvalidInput(format!(
                    "The start frequency {} MHz is not within the RF Explorer's frequency range of {}-{} MHz",
                    start.as_mhz_f64(),
                    min_max_freq.start().as_mhz_f64(),
                    min_max_freq.end().as_mhz_f64()
                )));
        } else if !min_max_freq.contains(&stop) {
            return Err(Error::InvalidInput(format!(
                    "The stop frequency {} MHz is not within the RF Explorer's frequency range of {}-{} MHz",
                    stop.as_mhz(),
                    min_max_freq.start().as_mhz_f64(),
                    min_max_freq.end().as_mhz_f64()
                )));
        }

        let min_max_span = active_model.min_span()..=active_model.max_span();
        if !min_max_span.contains(&(stop - start)) {
            return Err(Error::InvalidInput(format!(
                "The span {} MHz is not within the RF Explorer's span range of {}-{} MHz",
                (stop - start).as_mhz_f64(),
                min_max_span.start().as_mhz_f64(),
                min_max_span.end().as_mhz_f64()
            )));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), ret, err)]
    fn validate_min_max_amps(&self, min_amp_dbm: i16, max_amp_dbm: i16) -> Result<()> {
        // The bottom amplitude must be less than the top amplitude
        if min_amp_dbm >= max_amp_dbm {
            error!("");
            return Err(Error::InvalidInput(
                "The minimum amplitude must be less than the maximum amplitude".to_string(),
            ));
        }

        // The top and bottom amplitude must be within the RF Explorer's min and max amplitude range
        if !Self::MIN_MAX_AMP_RANGE_DBM.contains(&min_amp_dbm) {
            return Err(Error::InvalidInput(format!(
                "The amplitude {} dBm is not within the RF Explorer's amplitude range of {}-{} dBm",
                min_amp_dbm,
                Self::MIN_MAX_AMP_RANGE_DBM.start(),
                Self::MIN_MAX_AMP_RANGE_DBM.end()
            )));
        } else if !Self::MIN_MAX_AMP_RANGE_DBM.contains(&max_amp_dbm) {
            return Err(Error::InvalidInput(format!(
                "The amplitude {} dBm is not within the RF Explorer's amplitude range of {}-{} dBm",
                max_amp_dbm,
                Self::MIN_MAX_AMP_RANGE_DBM.start(),
                Self::MIN_MAX_AMP_RANGE_DBM.end()
            )));
        }

        Ok(())
    }
}

impl Deref for SpectrumAnalyzer {
    type Target = RfExplorer<MessageContainer>;
    fn deref(&self) -> &Self::Target {
        &self.rfe
    }
}

#[derive(Default)]
pub struct MessageContainer {
    pub(crate) config: (Mutex<Option<Config>>, Condvar),
    pub(crate) config_callback: Mutex<Callback<Config>>,
    pub(crate) sweep: (Mutex<Option<Sweep>>, Condvar),
    pub(crate) sweep_callback: Mutex<Callback<Sweep>>,
    pub(crate) screen_data: (Mutex<Option<ScreenData>>, Condvar),
    pub(crate) dsp_mode: (Mutex<Option<DspMode>>, Condvar),
    pub(crate) tracking_status: (Mutex<Option<TrackingStatus>>, Condvar),
    pub(crate) input_stage: (Mutex<Option<InputStage>>, Condvar),
    pub(crate) setup_info: (Mutex<Option<SetupInfo>>, Condvar),
    pub(crate) serial_number: (Mutex<Option<SerialNumber>>, Condvar),
}

impl crate::common::MessageContainer for MessageContainer {
    type Message = super::Message;

    fn cache_message(&self, message: Self::Message) {
        match message {
            Self::Message::Config(config) => {
                *self.config.0.lock().unwrap() = Some(config);
                self.config.1.notify_one();
                if let Some(ref mut cb) = *self.config_callback.lock().unwrap() {
                    cb(config);
                }
            }
            Self::Message::Sweep(sweep) => {
                *self.sweep.0.lock().unwrap() = Some(sweep);
                self.sweep.1.notify_one();
                if let Some(ref mut cb) = *self.sweep_callback.lock().unwrap() {
                    if let Some(ref sweep) = *self.sweep.0.lock().unwrap() {
                        cb(sweep.clone());
                    }
                }
            }
            Self::Message::ScreenData(screen_data) => {
                *self.screen_data.0.lock().unwrap() = Some(screen_data);
                self.screen_data.1.notify_one();
            }
            Self::Message::DspMode(dsp_mode) => {
                *self.dsp_mode.0.lock().unwrap() = Some(dsp_mode);
                self.dsp_mode.1.notify_one();
            }
            Self::Message::InputStage(input_stage) => {
                *self.input_stage.0.lock().unwrap() = Some(input_stage);
                self.input_stage.1.notify_one();
            }
            Self::Message::TrackingStatus(tracking_status) => {
                *self.tracking_status.0.lock().unwrap() = Some(tracking_status);
                self.tracking_status.1.notify_one();
            }
            Self::Message::SerialNumber(serial_number) => {
                *self.serial_number.0.lock().unwrap() = Some(serial_number);
                self.serial_number.1.notify_one();
            }
            Self::Message::SetupInfo(setup_info) => {
                *self.setup_info.0.lock().unwrap() = Some(setup_info);
                self.setup_info.1.notify_one();
            }
        }
    }

    fn wait_for_device_info(&self) -> bool {
        let (config_lock, config_cvar) = &self.config;
        let (setup_info_lock, setup_info_cvar) = &self.setup_info;

        // Check to see if we've already received a Config and SetupInfo
        if config_lock.lock().unwrap().is_some() && setup_info_lock.lock().unwrap().is_some() {
            return true;
        }

        // Wait to see if we receive a Config and SetupInfo before timing out
        config_cvar
            .wait_timeout_while(
                config_lock.lock().unwrap(),
                RfExplorer::<MessageContainer>::RECEIVE_INITIAL_DEVICE_INFO_TIMEOUT,
                |config| config.is_none(),
            )
            .unwrap()
            .0
            .is_some()
            && setup_info_cvar
                .wait_timeout_while(
                    setup_info_lock.lock().unwrap(),
                    RfExplorer::<MessageContainer>::RECEIVE_INITIAL_DEVICE_INFO_TIMEOUT,
                    |setup_info| setup_info.is_none(),
                )
                .unwrap()
                .0
                .is_some()
    }
}

impl RfExplorerMessageContainer for MessageContainer {
    type Model = super::Model;

    fn setup_info(&self) -> Option<SetupInfo<Self::Model>> {
        self.setup_info.0.lock().unwrap().clone()
    }

    fn screen_data(&self) -> Option<ScreenData> {
        self.screen_data.0.lock().unwrap().clone()
    }
}

impl Debug for MessageContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageContainer")
            .field("config", &self.config.0.lock().unwrap())
            .field("sweep", &self.sweep.0.lock().unwrap())
            .field("screen_data", &self.screen_data.0.lock().unwrap())
            .field("dsp_mode", &self.dsp_mode.0.lock().unwrap())
            .field("tracking_status", &self.tracking_status.0.lock().unwrap())
            .field("input_stage", &self.input_stage.0.lock().unwrap())
            .field("setup_info", &self.setup_info.0.lock().unwrap())
            .field("serial_number", &self.serial_number.0.lock().unwrap())
            .finish()
    }
}