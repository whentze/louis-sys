#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use std::sync::atomic::{AtomicBool, Ordering};

// Whether or not we (the -sys crate) hold the token right now.
static HAVE_TOKEN : AtomicBool = AtomicBool::new(true);

/// A token that can only ever exist once in the context of an entire process.
/// This is needed since liblouis is inherently thread-unsafe.
/// Safe abstractions over this crate have to guard all liblouis function calls,
/// allowing them only if this token is held.
/// Note that the token itself does not enable any safe calling of liblouis functions - higher-level bindings need to provide this.
pub struct ThreadUnsafetyToken(());

impl ThreadUnsafetyToken {
    /// Tries to get an exclusive ThreadUnsafetyToken, returning it if successful and `None` otherwise.
    pub fn take() -> Option<Self> {
        if HAVE_TOKEN.swap(false, Ordering::SeqCst) {
            Some(ThreadUnsafetyToken(()))
        } else {
            None
        }
    }
}

impl Drop for ThreadUnsafetyToken {
    fn drop(&mut self) {
        if HAVE_TOKEN.swap(true, Ordering::SeqCst) {
            panic!("Tried to drop ThreadUnsafetyToken back but we're also holding one. Something is very wrong and UB is likely!")
        } else {
            ()
        }
    }
}


include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate semver_parser;