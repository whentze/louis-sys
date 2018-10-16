use super::*;


use semver_parser::version;
use std::{
    ffi::{CStr, CString},
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

// As liblouis is thread-unsafe, we need to restrict access to it to a single thread
// I'm using a Mutex and a convenience macro
// see https://github.com/rust-lang/rust/issues/43155#issuecomment-314279433
lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    static ref message_counter: AtomicUsize = AtomicUsize::new(0);
}
macro_rules! test {
    (fn $name:ident() $body:block) => {
        #[test]
        fn $name() {
            let _guard = TEST_MUTEX.lock().unwrap();
            {
                $body
            }
        }
    };
}

test!{
fn liblouis_version() {
    let version_str = unsafe { CStr::from_ptr(lou_version()) }.to_str().unwrap();
    assert!(
        version::parse(version_str).unwrap().major >= 3,
        "liblouis version is too old (< 3.0.0)"
    );
    println!("liblouis version: {}", version_str);
}}

test!{
fn liblouis_charsize() {
    let charsize = unsafe { lou_charSize() };
    assert!(
        charsize == 2 || charsize == 4,
        "liblouis character size is not 16 or 32 bits"
    );
    println!("liblouis character size: {} bytes", charsize);
}}

test!{
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
}}

test!{
fn liblouis_roundtrip2() {
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
            64,
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
}}
test!{
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
            64,
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
}}

test!{
fn liblouis_logging() {
    message_counter.store(0, Ordering::Relaxed);
    unsafe extern "C" fn log_callback(level: logLevels, message: *const ::std::os::raw::c_char) {
        println!(
            "log message from liblouis: level {}: {}",
            level,
            CStr::from_ptr(message).to_str().unwrap()
        );
        message_counter.fetch_add(1, Ordering::Relaxed);
    }
    // Install our custom stdout log callback and lower the log level to be more noisy
    unsafe {
        lou_registerLogCallback(Some(log_callback));
        lou_setLogLevel(logLevels_LOG_ALL);
    }
    let sentence = "Let's translate this sentence to test logging";

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
            64,
        )
    };
    assert_eq!(res, 1);
    assert!(
        message_counter.load(Ordering::Relaxed) > 0,
        "The custom logging callback was not called for some reason."
    );
    // Reset log level and callback
    unsafe {
        lou_registerLogCallback(None);
        lou_setLogLevel(logLevels_LOG_INFO);
    }
}}
