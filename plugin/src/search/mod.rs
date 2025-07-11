//! i.e. PCRE 2 mode
//!
//! https://www.pcre.org/original/doc/html/pcreposix.html

use core::str;
use std::{
    borrow::Cow,
    ffi::{CStr, c_char, c_void},
    slice,
};

use everything_plugin::log::*;
use ib_matcher::matcher::{IbMatcher, PinyinMatchConfig};

use crate::HANDLER;

#[unsafe(no_mangle)]
extern "C" fn search_compile(pattern: *const c_char, cflags: u32, modifiers: u32) -> *const c_void {
    let pattern = unsafe { CStr::from_ptr(pattern) }.to_string_lossy();
    debug!(?pattern, cflags, modifiers, "Compiling IbMatcher");
    let app = unsafe { HANDLER.app() };
    let matcher = IbMatcher::builder(pattern.as_ref())
        // TODO
        .is_pattern_partial(true)
        .pinyin(
            PinyinMatchConfig::builder(app.config.pinyin_search.notations())
                .data(&app.pinyin_data)
                // TODO
                // .case_insensitive(app.config.pinyin_search.)
                .allow_partial_pattern(false)
                .build(),
        )
        .maybe_romaji(app.romaji.clone())
        .analyze(true)
        .build();
    let r = Box::new(matcher);
    Box::into_raw(r) as _
}

#[allow(non_camel_case_types)]
type regoff_t = i32;

/// The structure in which a captured offset is returned.
#[allow(non_camel_case_types)]
#[repr(C)]
struct regmatch_t {
    rm_so: regoff_t,
    rm_eo: regoff_t,
}

#[unsafe(no_mangle)]
extern "C" fn search_exec(
    matcher: *const c_void,
    haystack: *const c_char,
    length: u32,
    nmatch: usize,
    pmatch: *mut regmatch_t,
    eflags: u32,
) -> i32 {
    let matcher = unsafe { &*(matcher as *const IbMatcher) };

    let haystack = unsafe { slice::from_raw_parts(haystack as _, length as usize) };
    let buf;
    let haystack = if cfg!(debug_assertions) {
        buf = String::from_utf8_lossy(haystack);
        if let Cow::Owned(_) = &buf {
            error!(?haystack, ?buf, "haystack invalid utf8");
        }
        buf.as_ref()
    } else {
        // TODO: Optimization
        // buf = String::from_utf8_lossy(haystack);
        // buf.as_ref()

        unsafe { str::from_utf8_unchecked(haystack) }
    };

    if let Some(m) = matcher.find(haystack) {
        if nmatch > 0 {
            unsafe {
                (*pmatch).rm_so = m.start() as _;
                (*pmatch).rm_eo = m.end() as _;
            }
            1
        } else {
            0
        }
    } else {
        -1
    }

    // pysse

    // found 1 files with 24 threads in 0.021186 seconds
    // found 0 folders with 24 threads in 0.002326 seconds

    // c++
    // found 2 files with 24 threads in 0.292198 seconds
    // found 0 folders with 24 threads in 0.032536 seconds
    // found 2 files with 24 threads in 0.036560 seconds
    // found 0 folders with 24 threads in 0.003925 seconds
    // found 2 files with 24 threads in 0.036331 seconds
    // found 0 folders with 24 threads in 0.003879 seconds
    // found 2 files with 24 threads in 0.036273 seconds
    // found 0 folders with 24 threads in 0.003875 seconds

    // pcre2
    // found 2 files with 24 threads in 0.298361 seconds
    // found 0 folders with 24 threads in 0.033580 seconds
    // found 2 files with 24 threads in 0.282134 seconds
    // found 0 folders with 24 threads in 0.031068 seconds

    // length
    // found 2 files with 24 threads in 0.172370 seconds
    // found 0 folders with 24 threads in 0.015033 seconds
    // found 2 files with 24 threads in 0.175381 seconds
    // found 0 folders with 24 threads in 0.015673 seconds
    // debug
    // found 2 files with 24 threads in 0.937618 seconds
    // found 0 folders with 24 threads in 0.077921 seconds

    // Box dyn
    // found 2 files with 24 threads in 0.136043 seconds
    // found 0 folders with 24 threads in 0.012813 seconds
    // found 2 files with 24 threads in 0.133395 seconds
    // found 0 folders with 24 threads in 0.012782 seconds

    // str::from_utf8_unchecked
    // found 2 files with 24 threads in 0.126010 seconds
    // found 0 folders with 24 threads in 0.011448 seconds

    // min_haystack_len
    // found 2 files with 24 threads in 0.130524 seconds
    // found 0 folders with 24 threads in 0.011800 seconds

    // sub_test is_ascii fast fail optimization

    // ac
    // found 2 files with 24 threads in 0.025432 seconds
    // found 0 folders with 24 threads in 0.003713 seconds

    // REG_STARTEND
    // found 2 files with 24 threads in 0.014172 seconds
    // found 0 folders with 24 threads in 0.001881 seconds
    // found 2 files with 24 threads in 0.013749 seconds
    // found 0 folders with 24 threads in 0.001856 seconds
    // np:
    // found 1 files with 24 threads in 0.008721 seconds
    // found 0 folders with 24 threads in 0.001185 seconds
}

#[unsafe(no_mangle)]
extern "C" fn search_free(matcher: *mut c_void) {
    drop(unsafe { Box::from_raw(matcher as *mut IbMatcher) });
}
