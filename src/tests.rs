use super::*;
use semver_parser::version;
use std::ffi::{CStr, CString};
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

#[test]
fn liblouis_tables() {
    println!("liblouis tables:");
    let mut offset = 0;
    unsafe {
        let list_begin = lou_listTables();
        loop {
            let ptr = *(list_begin.offset(offset));
            if ptr == std::ptr::null() {
                break;
            }
            let table_name = Path::new(CStr::from_ptr(ptr).to_str().unwrap())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            print!("{} ", table_name);
            offset += 1;
        }
    };
    println!("");
    assert!(offset > 0, "No tables were found.");
    println!("Found {} tables in total.", offset);
}

#[test]
fn liblouis_roundtrip() {
    // ASCII only so we don't have to deal with utf8/utf16 issues for now
    let sentence = "This is an example sentence.";

    let inbuf: Vec<widechar> = sentence.bytes().map(Into::into).collect();
    let mut inlen = inbuf.capacity() as std::os::raw::c_int;
    let mut outbuf: Vec<widechar> = Vec::with_capacity(50);
    let mut outlen = outbuf.capacity() as std::os::raw::c_int;

    let res = unsafe {
        lou_translateString(
            CString::new("en_US.tbl").unwrap().as_ptr(),
            inbuf.as_ptr(),
            &mut inlen as *mut _,
            outbuf.as_mut_ptr(),
            &mut outlen as *mut _,
            std::ptr::null::<formtype>() as *mut _,
            std::ptr::null::<std::os::raw::c_char>() as *mut _,
            0,
        )
    };
    assert_eq!(res, 1);

    let inbuf = unsafe { Vec::from_raw_parts(outbuf.as_mut_ptr(), outlen as usize, 50) };
    std::mem::forget(outbuf);
    let mut inlen = outlen;

    let mut outbuf: Vec<widechar> = Vec::with_capacity(50);
    let mut outlen = outbuf.capacity() as std::os::raw::c_int;

    let res = unsafe {
        lou_backTranslateString(
            CString::new("en_US.tbl").unwrap().as_ptr(),
            inbuf.as_ptr(),
            &mut inlen as *mut _,
            outbuf.as_mut_ptr(),
            &mut outlen as *mut _,
            std::ptr::null::<formtype>() as *mut _,
            std::ptr::null::<std::os::raw::c_char>() as *mut _,
            0,
        )
    };
    assert_eq!(res, 1);

    let finalbuf = unsafe { Vec::from_raw_parts(outbuf.as_mut_ptr(), outlen as usize, 50) };
    std::mem::forget(outbuf);

    let new_sentence = String::from_utf8(finalbuf.iter().map(|w| *w as u8).collect()).unwrap();
    assert_eq!(*sentence, *new_sentence);
}
