use crate::util;

pub fn error(msg: &str) {
    println!("[{}][ERROR] {}", util::generate_timestamp(), msg);
}

pub fn info(msg: &str) {
    println!("[{}][INFO] {}", util::generate_timestamp(), msg);
}

pub fn warn(msg: &str) {
    println!("[{}][WARN] {}", util::generate_timestamp(), msg);
}

pub fn debug(msg: &str) {
    println!("[{}][DEBUG] {}", util::generate_timestamp(), msg);
}
