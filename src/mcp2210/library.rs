use std::borrow::Borrow;
use std::ffi::c_void;

use libloading::Library;

use crate::MCP2210Error;

static VENDOR_ID: u16 = 0x4D8;
static DEVICE_ID: u16 = 0xDE;

pub struct MCP2210Library {
    lib: Library,
    handle: c_void
}

impl MCP2210Library {
    pub fn new() -> Result<MCP2210Library, Box<dyn std::error::Error>> {
        unsafe {
            let lib = Library::new("mcp2210_dll_um_x64")?;
            let handle = MCP2210Library::connect(lib.borrow())?;
            let res = MCP2210Library { lib, handle };
            Ok(res)
        }
    }

    fn connect(lib: &Library) -> Result<c_void, Box<dyn std::error::Error>> {
        let device_index = 1;
        let device_path: &mut [u8] = &mut [];
        let device_path_size: &mut u64 = &mut 0;
        let device_handle: c_void;

        unsafe {
            let connect: libloading::Symbol<unsafe extern fn(u16, u16, u32, &mut [u8], &mut u64) -> c_void> = lib.get(b"Mcp2210_OpenByIndex")?;
            device_handle = connect(VENDOR_ID, DEVICE_ID, device_index, device_path, device_path_size);
        }

        if *device_path_size > 0 {
            let res = String::from_utf8(Vec::from(device_path))?;
            println!("Device Path: {}, size: {}", res, device_path_size);
        }

        Ok(device_handle)
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

        let error_code = MCP2210Error(match result {
            -20 => "Memory allocation error",
            -1 => "Unknown error",
            _ => "Unhandled error"
        }.into());

        Err(error_code)
    }

    //pub fn set_gpio_pin(&self) -> Result<(), MCP2210Error> {}
}