use std::borrow::Borrow;
use std::collections::HashMap;
use std::ffi::c_void;

use libloading::Library;
use num_derive::FromPrimitive;
use num;

use crate::MCP2210Error;

static VENDOR_ID: u16 = 0x4D8;
static DEVICE_ID: u16 = 0xDE;
static DEFAULT_DEVICE_INDEX: u32 = 0;
const GPIO_PINS: usize = 9;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum GPIODirection {
    Out = 0,
    In = 1,
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum GPIOPinValue {
    Off = 0,
    On = 1,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum GPIOWakeUp {
    Disabled = 0,
    Enabled = 1,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum InterruptCountingMode {
    None = 0,
    FallingEdges = 1,
    RisingEdges = 2,
    LowPulse = 3,
    HighPulse = 4,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum BusReleaseOption {
    Enabled = 0,
    Disabled = 1,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive)]
pub enum GPIOPins {
    GP0 = 0,
    GP1 = 1,
    GP2 = 2,
    GP3 = 3,
    GP4 = 4,
    GP5 = 5,
    GP6 = 6,
    GP7 = 7,
    GP8 = 8,
}

#[allow(dead_code)]
pub enum MCP2210MemorySource {
    Volatile = 0,
    NonVolatile = 1,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum MCP2210PinDesignation {
    GPIO = 0,
    CS = 1,
    FN = 2,
}

#[derive(Debug)]
struct MCP2210GPIOInfo {
    designation: MCP2210PinDesignation,
    output: GPIOPinValue,
    direction: GPIODirection,
}

#[derive(Debug)]
pub struct MCP2210Config {
    gpio_info: HashMap<GPIOPins, MCP2210GPIOInfo>,
    wake_up: GPIOWakeUp,
    interrupt_counting: InterruptCountingMode,
    bus_release: BusReleaseOption,
}

pub struct MCP2210Library {
    lib: Library,
    handle: *mut c_void,
}

impl MCP2210Library {
    pub fn new() -> Result<MCP2210Library, Box<dyn std::error::Error>> {
        unsafe {
            let lib = Library::new("mcp2210_dll_um_x64")?;
            let handle = MCP2210Library::connect(lib.borrow(), DEFAULT_DEVICE_INDEX)?;
            let res = MCP2210Library { lib, handle };
            Ok(res)
        }
    }

    fn connect(lib: &Library, device_index: u32) -> Result<*mut c_void, MCP2210Error> {
        let device_path: &mut [u8] = &mut [];
        let device_path_size: &mut u64 = &mut 0;
        let device_handle: *mut c_void;

        unsafe {
            let connect: libloading::Symbol<unsafe extern fn(u16, u16, u32, &mut [u8], &mut u64) -> *mut c_void> = lib.get(b"Mcp2210_OpenByIndex")?;
            device_handle = connect(VENDOR_ID, DEVICE_ID, device_index, device_path, device_path_size);
        }
        let result_code: *mut i32 = device_handle.cast();

        if result_code as i32 == -1 {
            let error_info = MCP2210Library::get_last_error(lib).unwrap_or_else(|e| e);
            return Err(MCP2210Error(format!("Communication error with MCP2210, with handle: {:?}, message: {}", result_code, error_info)));
        }

        if *device_path_size > 0 {
            let res = String::from_utf8(Vec::from(device_path));
            if res.is_err() {
                return Err(MCP2210Error("Unable to format device path string!".parse().unwrap()));
            }
            println!("Device Path: {}, size: {}", res.unwrap(), device_path_size);
        }

        Ok(device_handle)
    }

    fn get_error_code(code: i32) -> MCP2210Error {
        MCP2210Error(match code {
            0 => "No error".to_string(),
            -1 => "Unknown error".to_string(),
            -2 => "Invalid Parameter".to_string(),
            -3 => "Buffer too small".to_string(),
            -10 => "NULL pointer parameter".to_string(),
            -20 => "Memory allocation error".to_string(),
            -30 => "Invalid file handler use".to_string(),
            -100 => "Error find device".to_string(),
            -101 => "We tried to connect to a device with a non existent index".to_string(),
            -103 => "No device matching the provided criteria was found".to_string(),
            -104 => "Internal function buffer is too small".to_string(),
            -105 => "An error occurred when trying to get the device handle".to_string(),
            -106 => "Connection already opened".to_string(),
            -107 => "Connection close failed".to_string(),
            -108 => "no device found with the given serial number".to_string(),
            _ => format!("Unhandled error code {code}")
        }.into())
    }

    fn get_last_error(lib: &Library) -> Result<MCP2210Error, MCP2210Error> {
        let last_error_code: i32;
        unsafe {
            let get_last_error_code: libloading::Symbol<unsafe extern fn() -> i32> = lib.get(b"Mcp2210_GetLastError")?;
            last_error_code = get_last_error_code();
        }

        Ok(MCP2210Library::get_error_code(last_error_code))
    }

    pub fn connected_devices(&self) -> Result<i32, MCP2210Error> {
        let result: i32;
        unsafe {
            let connected_devices: libloading::Symbol<unsafe extern fn(u16, u16) -> i32> = self.lib.get(b"Mcp2210_GetConnectedDevCount")?;
            result = connected_devices(VENDOR_ID, DEVICE_ID);
        }
        if result.is_positive() {
            return Ok(result);
        }

        Err(MCP2210Library::get_error_code(result))
    }

    pub fn get_gpio_pin_directions(&self) -> Result<u32, MCP2210Error> {
        let gpio_pins: &mut u32 = &mut 0;
        let res_code;
        unsafe {
            let get_gpio_pins: libloading::Symbol<unsafe extern fn(*mut c_void, &mut u32) -> i32> = self.lib.get(b"Mcp2210_GetGpioPinDir")?;
            res_code = get_gpio_pins(self.handle, gpio_pins);
        }

        if gpio_pins > &mut 0 {
            return Ok(*gpio_pins);
        }

        Err(MCP2210Library::get_error_code(res_code))
    }

    pub fn set_gpio_pin_directions(&self, pin: GPIOPins, direction: GPIODirection) -> Result<(), MCP2210Error> {
        let directions = self.get_gpio_pin_directions()?;
        let set_gpio_direction: libloading::Symbol<unsafe extern fn(*mut c_void, u32) -> i32>;

        println!("Directions: {directions}");
        unsafe {
            set_gpio_direction = self.lib.get(b"Mcp2210_SetGpioPinDir")?;
        }

        match direction {
            GPIODirection::In =>
                unsafe { set_gpio_direction(self.handle, directions & !(1 << pin as u32)) },
            GPIODirection::Out =>
                unsafe { set_gpio_direction(self.handle, directions | (1 << pin as u32)) }
        };
        Ok(())
    }

    pub fn get_gpio_pin_value(&self, pin: &GPIOPins) -> Result<GPIOPinValue, MCP2210Error> {
        let result: &mut u32 = &mut 0;
        let return_value: i32;
        unsafe {
            let gpio_pin_value: libloading::Symbol<unsafe extern fn(*mut c_void, &mut u32) -> i32> = self.lib.get(b"Mcp2210_GetGpioPinVal")?;
            return_value = gpio_pin_value(self.handle, result);
        }
        if return_value.is_negative() {
            return Err(MCP2210Library::get_error_code(return_value));
        }

        let val = *result & (1 << *pin as u32);

        Ok(match val {
            0 => GPIOPinValue::Off,
            _ => GPIOPinValue::On
        })
    }

    pub fn set_gpio_pin_value(&self, pin: GPIOPins, pin_value: GPIOPinValue) -> Result<(), MCP2210Error> {
        let value = self.get_gpio_pin_value(pin.borrow())?;
        let set_gpio_value: libloading::Symbol<unsafe extern fn(*mut c_void, u32, &mut u32) -> i32>;
        let new_pin_values: &mut u32 = &mut 0;

        println!("Value: {:?}", value);
        unsafe {
            set_gpio_value = self.lib.get(b"Mcp2210_SetGpioPinVal")?;
        }

        let result = match pin_value {
            GPIOPinValue::Off =>
                unsafe { set_gpio_value(self.handle, value as u32 & !(1 << pin as u32), new_pin_values) },
            GPIOPinValue::On =>
                unsafe { set_gpio_value(self.handle, value as u32 | (1 << pin as u32), new_pin_values) }
        };

        println!("Result: {result} {new_pin_values} {}", value as u32 | (1 << pin as u32));
        Ok(())
    }

    pub fn get_gpio_config(&self, from: MCP2210MemorySource) -> Result<MCP2210Config, MCP2210Error> {
        let p_gpio_pin_des: [u8; GPIO_PINS] = Default::default();
        let pdflt_gpio_output: &mut u32 = &mut 0;
        let pdflt_gpio_dir: &mut u32 = &mut 0;
        let prmt_wkup_en: &mut u8 = &mut 0;
        let pint_pin_md: &mut u8 = &mut 0;
        let pspi_bus_rel_en: &mut u8 = &mut 0;
        let return_value: i32;

        unsafe {
            let get_gpio_config: libloading::Symbol<unsafe extern fn(*mut c_void, u8, [u8; 9], &mut u32, &mut u32, &mut u8, &mut u8, &mut u8) -> i32>;
            get_gpio_config = self.lib.get(b"Mcp2210_GetGpioConfig")?;
            return_value = get_gpio_config(self.handle, from as u8, p_gpio_pin_des, pdflt_gpio_output, pdflt_gpio_dir, prmt_wkup_en, pint_pin_md, pspi_bus_rel_en);
        }

        if return_value.is_negative() {
            return Err(MCP2210Library::get_error_code(return_value));
        }

        let mut map = HashMap::new();
        for i in 0..GPIO_PINS {
            let designation = num::FromPrimitive::from_u8(p_gpio_pin_des[i]).unwrap();
            let output = num::FromPrimitive::from_u32((*pdflt_gpio_output >> i) & 1).unwrap();
            let direction = num::FromPrimitive::from_u32((*pdflt_gpio_dir >> i) & 1).unwrap();

            map.insert(num::FromPrimitive::from_usize(i).unwrap(), MCP2210GPIOInfo {
                designation,
                output,
                direction
            });
        };
        let wake_up = num::FromPrimitive::from_u8(*prmt_wkup_en ).unwrap();
        let interrupt_counting = num::FromPrimitive::from_u8(*pint_pin_md ).unwrap();
        let bus_release = num::FromPrimitive::from_u8(*pspi_bus_rel_en).unwrap();

        Ok(MCP2210Config { gpio_info: map, wake_up, interrupt_counting, bus_release })
    }
}