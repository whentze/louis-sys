#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
extern crate semver_parser;

#[cfg(test)]
mod tests;