use std::ffi::CString;
use libc::{c_void, execvp, mount, perror};

pub extern "C" fn init_process(command: *mut c_void) -> i32 {
    let cstr: CString = unsafe { CString::from_raw(command as *mut i8) };
    let proc = cstr.into_string().expect("Init: receive command failed");
    println!("{}", proc);
    unsafe {
        println!("pid {}", libc::getpid());
        let ret = mount("proc\0".as_ptr() as *const i8, "/proc\0".as_ptr() as *const i8, "proc\0".as_ptr() as *const i8, 0, std::ptr::null());
        if ret == -1 {
            println!("Error: mount failed");
            perror(std::ptr::null());
            return -1;
        }
        let argv = [std::ptr::null()];
        let ret = execvp(command as *const i8, argv.as_ptr() as *const *const i8);
        if ret == -1 {
            println!("Error: exec failed");
            perror(std::ptr::null());
            return -1;
        }
    }
    return 0;
}