use std::process;

use crate::mcp2210::errors::MCP2210Error;
use crate::mcp2210::library::{GPIODirection, GPIOPins, MCP2210Library};

mod mcp2210 {
    pub mod errors;
    pub mod library;
}

fn main() {
    let mcp2210 = MCP2210Library::new().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let result = mcp2210.connected_devices();
    match result {
        Ok(e) => println!("Found {} devices", e),
        Err(e) => {
            println!("Error: {}", e)
        }
    }

    mcp2210.set_gpio_pin(GPIOPins::GP1, GPIODirection::IN).expect("Error while setting GPIO pin");
}
