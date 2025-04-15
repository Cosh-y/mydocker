use log::error;
use libc::perror;

// 使用 #[macro_export] 属性将宏导出到 crate 根目录
#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

pub fn check_libc_ret(ret: i32, msg: &str) {
    if ret < 0 {
        error!("Error: {}", msg);
        unsafe { perror(std::ptr::null()) };
        std::process::exit(1);
    }
}
