use std::process;

use crate::mcp2210::errors::MCP2210Error;
use crate::mcp2210::library::{GPIODirection, GPIOPins, GPIOPinValue, MCP2210Library, MCP2210MemorySource};

mod mcp2210 {
    pub mod errors;
    pub mod library;
}

fn main() {
    let mcp2210 = MCP2210Library::new().unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {}", err);
        process::exit(1);
    });

    let result = mcp2210.connected_devices();
    match result {
        Ok(e) => println!("Found {} devices", e),
        Err(e) => {
            println!("Error: {}", e)
        }
    }

    mcp2210.set_gpio_pin_directions(GPIOPins::GP1, GPIODirection::In)
        .expect("Error while setting GPIO pin");
    mcp2210.set_gpio_pin_directions(GPIOPins::GP2, GPIODirection::In)
        .expect("Error while setting GPIO pin");
    mcp2210.set_gpio_pin_directions(GPIOPins::GP3, GPIODirection::In)
        .expect("Error while setting GPIO pin");

    //mcp2210.set_gpio_pin_value(GPIOPins::GP1, GPIOPinValue::On).expect("Failed to set PIN value");
    let res = mcp2210.get_gpio_config(MCP2210MemorySource::Volatile).expect("Failed to load device config");

    println!("{:#?}", res);
}
