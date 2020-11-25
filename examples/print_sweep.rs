use rfe::{self, RfExplorer};

fn main() {
    if let Some(mut spectrum_analyzer) = rfe::first_spectrum_analyzer() {
        println!("{:?}", spectrum_analyzer.request_config());
        println!("{:?}", spectrum_analyzer.get_sweep());
    } else {
        println!("No spectrum analyzers connected");
    }
}
