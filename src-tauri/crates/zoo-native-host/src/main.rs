//! zoo-native-host: started by Chrome or Edge for the Zoo Downloader extension.
//! stdout is the browser's channel; diagnostics go to stderr (the browser logs it).

use std::io;
use std::process::ExitCode;
use zoo_native_host::host::{is_our_extension, read_message, write_message, Bridge};

fn main() -> ExitCode {
    if !is_our_extension(std::env::args().skip(1)) {
        eprintln!("zoo-native-host: only the Zoo Downloader extension may start this program");
        return ExitCode::FAILURE;
    }
    let bridge = match Bridge::next_to_this_exe() {
        Ok(bridge) => bridge,
        Err(e) => {
            eprintln!("zoo-native-host: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (mut input, mut output) = (io::stdin().lock(), io::stdout().lock());
    loop {
        match read_message(&mut input) {
            Ok(Some(request)) => {
                if let Err(e) = write_message(&mut output, bridge.forward(&request).as_bytes()) {
                    eprintln!("zoo-native-host: {e}");
                    return ExitCode::FAILURE;
                }
            }
            Ok(None) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("zoo-native-host: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
}
