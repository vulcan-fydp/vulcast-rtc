#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate link_cplusplus;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        unsafe {
            super::init();
            super::hello();
        }
    }
}
