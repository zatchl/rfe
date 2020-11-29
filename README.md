# rfe

`rfe` is a library for communicating with RF Explorer spectrum analyzers and signal generators.

## Example

``` rust
fn main() {
    if let Some(mut spectrum_analyzer) = rfe::first_spectrum_analyzer() {
        println!("{:?}", spectrum_analyzer);
        println!("{:?}", spectrum_analyzer.get_sweep());
    } else {
        println!("No spectrum analyzers connected");
    }
}
```

## Setup

``` toml
# Cargo.toml
[dependencies]
rfe = { git = "https://github.com/zatchl/rfe" }
```

### Windows and macOS

* Download and install the appropriate [Silicon Labs CP210x USB driver](https://www.silabs.com/products/development-tools/software/usb-to-uart-bridge-vcp-drivers)

### Linux

* The Silicon Labs CP210x driver is included in the kernel
* Make sure the current user has read/write access to the RF Explorer
  * `ls -l /dev/ttyUSB*` to find the RF Explorer's group owner
  * `groups` to see if the current user is a member of the group
  * `gpasswd -a <user> <group>` to add the user to the group
