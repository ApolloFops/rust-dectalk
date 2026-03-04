#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// This may be a hacky workaround but IDK enough about rust to know if it is
// It does work though
#[link(name = "dectalk")]
unsafe extern "C" {}
