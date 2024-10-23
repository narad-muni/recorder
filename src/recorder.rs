use bus::{Bus, BusReader};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::Error;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{fs, thread};

use crate::constants::BUF_SIZE;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[serde(rename = "*")]
    All,
    TcpClient,
    TcpServer,
    TcpProxy,
    File,
    Udp,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Block {
    #[serde(default = "default_ip")]
    pub source_ip: String,
    #[serde(default = "default_port")]
    pub source_port: u16,
    #[serde(default = "default_ip")]
    pub bind_ip: String,
    #[serde(default = "default_port")]
    pub bind_port: u16,
    #[serde(default)]
    pub file_path: String,
    #[serde(default)]
    pub interface_ip: String,
    #[serde(default)]
    pub no_headers: bool,
    #[serde(default)]
    pub play_timed: bool,
    #[serde(default)]
    pub play_loop: bool,
    #[serde(default)]
    pub controlled_play: bool,
    #[serde(default = "default_speed")]
    pub speed_multiplier: f64,
    pub mode: Mode,
}

fn default_speed() -> f64 {
    1.0
}

fn default_ip() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    0
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub inputs: Vec<Block>,
    pub outputs: Vec<Block>,
    pub from: Vec<Mode>,
    pub to: Vec<Mode>,
}

pub struct Recorder {
    bus: Arc<Mutex<Bus<([u8; BUF_SIZE], u32)>>>,
    output_bus: Vec<Arc<Mutex<BusReader<([u8; BUF_SIZE], u32)>>>>,
    input: Vec<(Block, Arc<dyn Input>)>,
    output: Vec<(Block, Arc<dyn Output>)>,
}

#[derive(Debug)]
pub enum AdapterType {
    Input(Arc<dyn Input>),
    Output(Arc<dyn Output>),
}

impl<'a> Recorder {
    pub fn new(config_path: String, adapters: Vec<(Mode, AdapterType)>) -> Recorder {
        let config = fs::read_to_string(config_path).unwrap();
        let settings: Settings = serde_json::from_str(&config).unwrap();

        // Filter out adapters selected in settings
        let input_adapters: Vec<(Block, Arc<dyn Input>)> = settings
            .inputs
            .into_iter()
            .filter(|i| settings.from.contains(&i.mode) || settings.from.contains(&Mode::All))
            .filter_map(|i| {
                let (_, adapter) = adapters
                    .iter()
                    .find(|(mode, adapter)| match adapter {
                        AdapterType::Input(_) => mode == &i.mode,
                        _ => false,
                    })
                    .expect(format!("Cannot find input adapter for {:?}", i.mode).as_str());

                if let AdapterType::Input(adapter) = adapter {
                    Some((i, adapter.clone()))
                } else {
                    None
                }
            })
            .collect();

        // Filter out adapters selected in settings
        let output_adapters: Vec<(Block, Arc<dyn Output>)> = settings
            .outputs
            .into_iter()
            .filter(|i| settings.to.contains(&i.mode) || settings.to.contains(&Mode::All))
            .filter_map(|i| {
                let (_, adapter) = adapters
                    .iter()
                    .find(|(mode, adapter)| match adapter {
                        AdapterType::Output(_) => mode == &i.mode,
                        _ => false,
                    })
                    .expect(format!("Cannot find output adapter for {:?}", i.mode).as_str());

                if let AdapterType::Output(adapter) = adapter {
                    Some((i, adapter.clone()))
                } else {
                    None
                }
            })
            .collect();

        if input_adapters.len() == 0 {
            panic!("Error, No input adapters found");
        }

        // if no output adapters and no proxy adapter, then error
        if output_adapters.len() == 0 && !input_adapters.iter().any(|(block, _)| block.mode == Mode::TcpProxy) {
            panic!("Error, No output adapters found");
        }

        // Create a bus of buffer for input
        let mut bus = Bus::<([u8; BUF_SIZE], u32)>::new(1000);
        // Create a output vector of output bus
        let mut output_bus = vec![];

        // For each output adapter create one output bus
        for _ in 0..output_adapters.len() {
            output_bus.push(Arc::new(Mutex::new(bus.add_rx())));
        }

        println!("Initializing recorder");
        print!("Inputs are ");
        for (i, _) in input_adapters.iter() {
            print!("{:?} ", i.mode)
        }

        print!("\nOutputs are ");
        for (i, _) in output_adapters.iter() {
            print!("{:?} ", i.mode)
        }

        Recorder {
            bus: Arc::new(Mutex::new(bus)),
            output_bus,
            input: input_adapters,
            output: output_adapters,
        }
    }

    /// Read function spawns a reader in a thread and returns handle to the thread
    pub fn read(&self) -> JoinHandle<()> {
        let inputs = self.input.clone();
        let bus = self.bus.clone();

        thread::spawn(move || {
            let mut bus = bus.lock().unwrap();

            for (source, input) in inputs {
                input.read(source, &mut bus).unwrap();
            }
        })
    }

    /// Write function spawns n threads for n output adapters and returns immediately
    pub fn write(&self) {
        let outputs = self.output.clone();

        let mut i = 0;

        for (source, output) in outputs {
            let output_bus = self.output_bus.get(i).unwrap().clone();

            i += 1;

            thread::spawn(move || {
                let mut output_bus = output_bus.lock().unwrap();

                output.write(source, &mut output_bus).unwrap();
            });
        }
    }
}

pub trait Input: Send + Sync + Debug {
    /// This function should read from the source depending on the implementation and write it to channel.
    /// Should read in blocking mode.
    /// Must only return in case of error.
    fn read(&self, block: Block, channel: &mut Bus<([u8; BUF_SIZE], u32)>) -> Result<(), Error>;
}

pub trait Output: Send + Sync + Debug {
    /// This function should read from the channel and write it to source depending on the implementation.
    /// Should write in blocking mode.
    /// Must only return in case of error.
    fn write(
        &self,
        block: Block,
        channel: &mut BusReader<([u8; BUF_SIZE], u32)>,
    ) -> Result<(), Error>;
}
