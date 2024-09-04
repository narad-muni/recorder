use std::{env, sync::Arc};

use adapters::{file_adapter::FileAdapter, tcp_adapter::TcpAdapter, udp_adapter::UdpAdapter};
use recorder::{AdapterType, Mode, Recorder};

mod adapters;
mod constants;
mod recorder;
mod utils;

fn main() {
    let config_path: String = env::args().nth(1).unwrap_or("settings.json".to_string());

    // Create all adapter instances
    let tcp_adapter = Arc::new(TcpAdapter {});
    let file_adapter = Arc::new(FileAdapter {});
    let udp_adapter = Arc::new(UdpAdapter {});

    // Register all adapters here
    let mapping: Vec<(Mode, AdapterType)> = vec![
        // Input adapters
        (Mode::Tcp, AdapterType::Input(tcp_adapter.clone())),
        (Mode::File, AdapterType::Input(file_adapter.clone())),
        (Mode::Udp, AdapterType::Input(udp_adapter.clone())),
        // Output adapters
        (Mode::Tcp, AdapterType::Output(tcp_adapter.clone())),
        (Mode::File, AdapterType::Output(file_adapter.clone())),
        (Mode::Udp, AdapterType::Output(udp_adapter.clone())),
    ];

    let recorder = Recorder::new(config_path, mapping);

    // First start writer thread
    recorder.write();
    // Start reader thread and then block on it
    recorder.read().join().unwrap();
}