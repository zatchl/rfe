use rfe;

fn main() {
    if let Some(spectrum_analyzer) = rfe::first_spectrum_analyzer() {
        println!("{:?}", spectrum_analyzer);
        println!("{:?}", spectrum_analyzer.last_sweep());
    } else {
        println!("No spectrum analyzers connected");
    }
}
