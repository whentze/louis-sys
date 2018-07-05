use super::*;
use semver_parser::version;
use std::ffi::CStr;
use std::path::Path;

#[test]
fn liblouis_version() {
    let version_str = unsafe { CStr::from_ptr(lou_version()) }.to_str().unwrap();
    assert!(
        version::parse(version_str).unwrap().major >= 3,
        "liblouis version is too old (< 3.0.0)"
    );
    println!("liblouis version: {}", version_str);
}

#[test]
fn liblouis_charsize() {
    let charsize = unsafe { lou_charSize() };
    assert!(
        charsize == 2 || charsize == 4,
        "liblouis character size is not 16 or 32 bits"
    );
    println!("liblouis character size: {} bytes", charsize);
}