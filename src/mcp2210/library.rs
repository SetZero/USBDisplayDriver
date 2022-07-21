use libloading::Library;
use crate::MCP2210Error;

pub struct MCP2210Library {
    lib: Library,
}

impl MCP2210Library {
    pub fn new() -> Result<MCP2210Library, Box<dyn std::error::Error>> {
        unsafe {
            let lib = Library::new("mcp2210_dll_um_x64")?;
            Ok(MCP2210Library { lib })
        }
    }

    pub fn connected_devices(&self) -> Result<i32, MCP2210Error> {
        let result: i32;
        unsafe {
            let connected_devices: libloading::Symbol<unsafe extern fn(u16, u16) -> i32> = self.lib.get(b"Mcp2210_GetConnectedDevCount")?;
            result = connected_devices(0x4D8, 0xDE);
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
}