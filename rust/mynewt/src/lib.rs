//! Mynewt API for Rust. Contains Rust bindings for Mynewt API for C, generated by `bindgen`.
//! Also includes safe versions of Mynewt APIs created specially for Rust.

#![no_std]                        //  Don't link with standard Rust library, which is not compatible with embedded systems
#![feature(const_transmute)]      //  Allow `transmute` for initialising Mynewt structs
#![feature(trace_macros)]         //  Enable tracing of macros
#![feature(proc_macro_hygiene)]   //  Allow proc macros to be unhygienic
#![feature(custom_attribute)]     //  Allow Custom Attributes like `#[safe_wrap]`

extern crate macros as mynewt_macros;  //  Import Procedural Macros from `macros` library

#[allow(non_camel_case_types)]    //  Allow type names to have non-camel case
#[allow(non_upper_case_globals)]  //  Allow globals to have lowercase letters
pub mod kernel;                   //  Mynewt Kernel API. Export folder `kernel` as Rust module `mynewt::kernel`

#[allow(dead_code)]               //  Suppress warnings of unused constants and vars
#[allow(non_camel_case_types)]    //  Allow type names to have non-camel case
#[allow(non_upper_case_globals)]  //  Allow globals to have lowercase letters
pub mod hw;                       //  Mynewt Hardware API. Export folder `hw` as Rust module `mynewt::hw`

#[allow(dead_code)]               //  Suppress warnings of unused constants and vars
pub mod sys;                      //  Mynewt System API. Export folder `sys` as Rust module `mynewt::sys`

#[macro_use]                      //  Allow macros from Rust module `encoding`
#[allow(non_camel_case_types)]    //  Allow type names to have non-camel case
#[allow(non_upper_case_globals)]  //  Allow globals to have lowercase letters
pub mod encoding;                 //  Mynewt Encoding API. Export folder `encoding` as Rust module `mynewt::encoding`

#[macro_use]                      //  Allow macros from Rust module `util`
pub mod util;                     //  Mynewt Utility API. Export folder `encoding` as Rust module `mynewt::util`

#[allow(non_camel_case_types)]    //  Allow type names to have non-camel case
#[allow(non_upper_case_globals)]  //  Allow globals to have lowercase letters
pub mod libs;                     //  Mynewt Custom API. Export folder `libs` as Rust module `mynewt::libs`

///  Initialise the Mynewt system.  Start the Mynewt drivers and libraries.  Equivalent to `sysinit()` macro in C.
pub fn sysinit() {
    unsafe { rust_sysinit(); }
    sys::console::flush();
}

/// Return type and error codes for Mynewt API
pub mod result {
    use crate::kernel::os;

    /// Common return type for Mynewt API.  If no error, returns `Ok(val)` where val has type T.
    /// Upon error, returns `Err(err)` where err is the MynewtError error code.
    pub type MynewtResult<T> = ::core::result::Result<T, MynewtError>;

    /// Error codes for Mynewt API
    #[repr(i32)]
    #[derive(PartialEq)]
    #[allow(non_camel_case_types)]    //  Allow type names to have non-camel case
    pub enum MynewtError {
        /// Error code 0 means no error.
        SYS_EOK         = os::SYS_EOK as i32,
        SYS_ENOMEM      = os::SYS_ENOMEM,
        SYS_EINVAL      = os::SYS_EINVAL,
        SYS_ETIMEOUT    = os::SYS_ETIMEOUT,
        SYS_ENOENT      = os::SYS_ENOENT,
        SYS_EIO         = os::SYS_EIO,
        SYS_EAGAIN      = os::SYS_EAGAIN,
        SYS_EACCES      = os::SYS_EACCES,
        SYS_EBUSY       = os::SYS_EBUSY,
        SYS_ENODEV      = os::SYS_ENODEV,
        SYS_ERANGE      = os::SYS_ERANGE,
        SYS_EALREADY    = os::SYS_EALREADY,
        SYS_ENOTSUP     = os::SYS_ENOTSUP,
        SYS_EUNKNOWN    = os::SYS_EUNKNOWN,
        SYS_EREMOTEIO   = os::SYS_EREMOTEIO,
        SYS_EDONE       = os::SYS_EDONE,
        SYS_EPERUSER    = os::SYS_EPERUSER,
    }

    /// Cast `MynewtError` to `i32`
    impl From<MynewtError> for i32 {
        /// Cast `MynewtError` to `i32`
        fn from(err: MynewtError) -> Self {
            err as i32
        }
    }

    /// Cast `i32` to `MynewtError`
    impl From<i32> for MynewtError {
        /// Cast `i32` to `MynewtError`
        fn from(num: i32) -> Self {
            unsafe { 
                ::core::mem::transmute::
                    <i32, MynewtError>
                    (num)
            }  
        }
    }

    /// Cast `()` to `MynewtError`
    impl From<()> for MynewtError {
        /// Cast `()` to `MynewtError`
        fn from(_: ()) -> Self {
            MynewtError::SYS_EUNKNOWN
        }
    }

    /// Implement formatted output for MynewtError
    impl core::fmt::Debug for MynewtError {
        fn fmt(&self, _fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            //  TODO
            Ok(())
        }
    }
}

/// Represents a null-terminated string, suitable for passing to Mynewt APIs as `* const char`.
/// The string could be a null-terminated byte string created in Rust, or a pointer to a null-terminated string returned by C.
/// Pointer may be null.
#[derive(Clone, Copy)]  //  Strn may be copied
pub struct Strn {
    /// Either a byte string terminated with null, or a pointer to a null-terminated string
    pub rep: StrnRep
}

/// Either a byte string or a string pointer
#[derive(Clone, Copy)]  //  StrnRep may be copied
#[repr(u8)]
pub enum StrnRep {
    /// Byte string terminated with null
    ByteStr(&'static [u8]),
    /// Pointer to a null-terminated string
    CStr(*const u8),
}

impl Strn {
    /// Create a new `Strn` with a byte string. Fail if the last byte is not zero.
    /// ```
    /// Strn::new(b"network\0")
    /// strn!("network")
    /// ```
    pub fn new(bs: &'static [u8]) -> Strn {
        assert_eq!(bs.last(), Some(&0u8), "no null");  //  Last byte must be 0.
        Strn { 
            rep: StrnRep::ByteStr(bs)
        }
    }

    /// Create a new `Strn` with a null-terminated string pointer returned by C.
    pub fn from_cstr(cstr: *const u8) -> Strn {
        Strn { 
            rep: StrnRep::CStr(cstr)
        }
    }

    /// Return a pointer to the string
    pub fn as_ptr(&self) -> *const u8 {
        match self.rep {
            StrnRep::ByteStr(bs) => { bs.as_ptr() }
            StrnRep::CStr(cstr)  => { cstr }
        }
    }

    /// Return the length of the string, excluding the terminating null. For safety, we limit to 128.
    pub fn len(&self) -> usize {
        match self.rep {
            StrnRep::ByteStr(bs) => { 
                assert_eq!(bs.last(), Some(&0u8), "no null");  //  Last byte must be 0.
                bs.len() - 1  //  Don't count the terminating null.
            }
            StrnRep::CStr(cstr)  => { 
                //  Look for the null termination.
                if cstr.is_null() { return 0; }
                for len in 0..127 {
                    let ptr: *const u8 =  ((cstr as u32) + len) as *const u8;
                    if unsafe { *ptr } == 0 { return len as usize; }                    
                }
                assert!(false, "big strn");  //  String too long
                return 128 as usize;
            }
        }
    }

    /// Return true if the string is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the byte string as a null-terminated `* const char` C-style string.
    /// Fail if the last byte is not zero.
    pub fn as_cstr(&self) -> *const u8 {
        match self.rep {
            StrnRep::ByteStr(bs) => { 
                assert_eq!(bs.last(), Some(&0u8), "no null");  //  Last byte must be 0.
                bs.as_ptr() as *const u8
            }
            StrnRep::CStr(cstr)  => { cstr }
        }
    }

    /// Return the byte string.
    /// Fail if the last byte is not zero.
    pub fn as_bytestr(&self) -> &'static [u8] {
        match self.rep {
            StrnRep::ByteStr(bs) => {                
                assert_eq!(bs.last(), Some(&0u8), "no null");  //  Last byte must be 0.
                &bs
            }
            StrnRep::CStr(_cstr)  => { 
                assert!(false, "strn cstr");  //  Not implemented
                b"\0"
            }
        }
    }

    /// Fail if the last byte is not zero.
    pub fn validate(&self) {
        match self.rep {
            StrnRep::ByteStr(bs) => {         
                assert_eq!(bs.last(), Some(&0u8), "no null");  //  Last byte must be 0.
            }
            StrnRep::CStr(_cstr)  => {}
        }
    }

    /// Fail if the last byte is not zero.
    pub fn validate_bytestr(bs: &'static [u8]) {
        assert_eq!(bs.last(), Some(&0u8), "no null");  //  Last byte must be 0.
    }
}

///  Allow threads to share Strn, since it is static.
unsafe impl Send for Strn {}

///  Allow threads to share Strn, since it is static.
unsafe impl Sync for Strn {}

///  Declare a pointer that will be used by C functions to return a value
pub type Out<T> = &'static mut T;

///  Declare a `void *` pointer that will be passed to C functions
pub type Ptr = *mut ::cty::c_void;

///  Declare a `NULL` pointer that will be passed to C functions
pub const NULL: Ptr = 0 as Ptr;

///  Import the custom interop helper library at `libs/mynewt_rust`
#[link(name = "libs_mynewt_rust")]  //  Functions below are located in the Mynewt build output `libs_mynewt_rust.a`
extern {
    ///  Initialise the Mynewt system.  Start the Mynewt drivers and libraries.  Equivalent to `sysinit()` macro in C.
    ///  C API: `void rust_sysinit()`
    fn rust_sysinit();  
}