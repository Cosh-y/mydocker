use libc::{c_void, read, execvp, mount, perror, MS_PRIVATE, MS_REC};
use log::{info, error};

use crate::run::RunArg;

pub extern "C" fn init_process(arg: *mut c_void) -> i32 {
    let run_arg_ref = unsafe { &*(arg as *mut RunArg) };
    info!("Init process started with args: image {} cpu {}", run_arg_ref.image, run_arg_ref.cpu.unwrap_or(0));
    
    unsafe {
        // let mut buffer = [0u8; 1024];
        // let bytes_read = read(3, buffer.as_mut_ptr() as *mut c_void, buffer.len());
        // if bytes_read <= 0 {
        //     error!("Error: failed to read from pipe");
        //     perror(std::ptr::null());
        //     return -1;
        // }
        // let arg = match CString::new(&buffer[..bytes_read as usize]) {
        //     Ok(cstr) => cstr,
        //     Err(_) => {
        //         error!("Error: invalid CString");
        //         return -1;
        //     }
        // };
        // info!("Read argument: {:?}", arg);

        if mount("\0".as_ptr() as *const i8, 
                "/\0".as_ptr() as *const i8,
                "\0".as_ptr() as *const i8,
                MS_PRIVATE | MS_REC,
                std::ptr::null()
            ) < 0 {
            error!("Error: mount failed");
            perror(std::ptr::null());
            return -1;
        }

        if mount("proc\0".as_ptr() as *const i8, 
                        "/proc\0".as_ptr() as *const i8, 
                        "proc\0".as_ptr() as *const i8, 
                        0, 
                        std::ptr::null()
                    ) < 0 {
            error!("Error: mount failed");
            perror(std::ptr::null());
            return -1;
        }

        let argv = [std::ptr::null()];
        let ret = execvp("/bin/bash\0".as_ptr() as *const i8, argv.as_ptr() as *const *const i8);
        if ret == -1 {
            error!("Error: exec failed");
            perror(std::ptr::null());
            return -1;
        }
    }
    return 0;
}