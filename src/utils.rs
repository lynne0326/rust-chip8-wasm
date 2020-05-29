extern crate getrandom;

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
}

pub fn get_random_buf() -> Result<[u8; 1], getrandom::Error> {
    let mut buf = [0u8; 1];
    getrandom::getrandom(&mut buf)?;
    Ok(buf)
}