extern crate alloc;
extern crate std;

use alloc::{format, vec};
use core::ffi::c_void;
use core::ptr::null_mut;
use std::mem;
use std::mem::{align_of, size_of};
use sync_ptr::*;

#[test]
pub fn test_debug() {
    let n = null_mut::<c_void>().as_sync_mut();
    assert_eq!(format!("{:?}", n), "SyncMutPtr(0x0)");
    let n = null_mut::<c_void>().as_sync_const();
    assert_eq!(format!("{:?}", n), "SyncConstPtr(0x0)");
    let n = null_mut::<c_void>().as_send_const();
    assert_eq!(format!("{:?}", n), "SendConstPtr(0x0)");
    let n = null_mut::<c_void>().as_send_mut();
    assert_eq!(format!("{:?}", n), "SendMutPtr(0x0)");

    #[cfg(target_pointer_width = "64")]
    {
        let n = null_mut::<c_void>().as_sync_mut();
        assert_eq!(
            format!("{:#?}", n),
            "SyncMutPtr(\n    0x0000000000000000,\n)"
        );
        let n = null_mut::<c_void>().as_sync_const();
        assert_eq!(
            format!("{:#?}", n),
            "SyncConstPtr(\n    0x0000000000000000,\n)"
        );
        let n = null_mut::<c_void>().as_send_const();
        assert_eq!(
            format!("{:#?}", n),
            "SendConstPtr(\n    0x0000000000000000,\n)"
        );
        let n = null_mut::<c_void>().as_send_mut();
        assert_eq!(
            format!("{:#?}", n),
            "SendMutPtr(\n    0x0000000000000000,\n)"
        );
    }

    #[cfg(target_pointer_width = "32")]
    {
        let n = null_mut::<c_void>().as_sync_mut();
        assert_eq!(format!("{:#?}", n), "SyncMutPtr(\n    0x00000000,\n)");
        let n = null_mut::<c_void>().as_sync_const();
        assert_eq!(format!("{:#?}", n), "SyncConstPtr(\n    0x00000000,\n)");
        let n = null_mut::<c_void>().as_send_const();
        assert_eq!(format!("{:#?}", n), "SendConstPtr(\n    0x00000000,\n)");
        let n = null_mut::<c_void>().as_send_mut();
        assert_eq!(format!("{:#?}", n), "SendMutPtr(\n    0x00000000,\n)");
    }
}

#[test]
pub fn test_cmp() {
    unsafe {
        let mut data = vec![0; 4096];
        let n = data.as_mut_ptr().as_sync_mut();
        let x = n.add(5).as_sync_mut();
        assert!(x > n);
        assert!(n < x);
        assert_ne!(n, x);
        assert_eq!(n, n);
        assert_eq!(x, x);
    }

    unsafe {
        let mut data = vec![0; 4096];
        let n = data.as_mut_ptr().as_send_mut();
        let x = n.add(5).as_send_mut();
        assert!(x > n);
        assert!(n < x);
        assert_ne!(n, x);
        assert_eq!(n, n);
        assert_eq!(x, x);
    }

    unsafe {
        let mut data = vec![0; 4096];
        let n = data.as_mut_ptr().as_send_const();
        let x = n.add(5).as_send_const();
        assert!(x > n);
        assert!(n < x);
        assert_ne!(n, x);
        assert_eq!(n, n);
        assert_eq!(x, x);
    }

    unsafe {
        let mut data = vec![0; 4096];
        let n = data.as_mut_ptr().as_sync_const();
        let x = n.add(5).as_sync_const();
        assert!(x > n);
        assert!(n < x);
        assert_ne!(n, x);
        assert_eq!(n, n);
        assert_eq!(x, x);
    }
}

#[test]
fn test() {
    unsafe {
        assert_eq!(size_of::<SyncConstPtr<c_void>>(), size_of::<*mut c_void>());
        assert_eq!(
            align_of::<SyncConstPtr<c_void>>(),
            align_of::<*mut c_void>()
        );

        let mut value = 45;
        let n: *mut u64 = &mut value;
        let x = n.as_sync_const();
        assert_eq!(x, x);
        let y = n.as_send_mut();
        let z = y.as_sync_const();
        let u = z.as_send_mut();
        assert_eq!(x, z);
        assert_eq!(u, u);

        assert_eq!(z.read(), 45);
        assert_eq!(z.inner().read(), 45);
        std::thread::spawn(move || assert!(!u.inner().is_null()))
            .join()
            .unwrap();
    }
}

struct RustControlStructureThatIsNowSend {
    some_handle: SendConstPtr<c_void>,
    some_rust_data: u64,
}

#[test]
fn example() {
    //Some handle or pointer obtained goes here.
    let handle: *mut c_void = null_mut();
    let data: u64 = 123u64;

    let rcs = RustControlStructureThatIsNowSend {
        some_handle: handle.as_send_const(),
        some_rust_data: data,
    };

    //Every *const T and *mut T has these fn's now.
    let _sync_const: SyncConstPtr<c_void> = handle.as_sync_const();
    let _send_const: SendConstPtr<c_void> = handle.as_send_const();

    //Every *mut T has these fn's now.
    let _sync_mut: SyncMutPtr<c_void> = handle.as_sync_mut();
    let _send_mut: SendMutPtr<c_void> = handle.as_send_mut();

    //The other Ptr types have the same constructors too.
    let _send_const_null: SendMutPtr<c_void> = SendMutPtr::null();
    let _send_const_new: SendMutPtr<c_void> = SendMutPtr::new(null_mut());
    let _from_address: SyncMutPtr<c_void> = SyncMutPtr::from_address(0usize);

    std::thread::spawn(move || {
        assert!(rcs.some_handle.is_null()); //Use boxed
        let _unwrapped: *const c_void = rcs.some_handle.inner(); //unwrap if you want
        let _unwrapped2: *const c_void = rcs.some_handle.into(); //Into<*const T> is also implemented. (*mut T too when applicable)
        let _address: usize = rcs.some_handle.as_address(); //Into<usize> is also implemented.
        let casted: SendConstPtr<usize> = rcs.some_handle.cast::<usize>(); //Cast if you want.
        unsafe {
            if !casted.is_null() {
                //In this example this is obviously always null...
                casted.read_volatile(); //Read if you want
            }
        }

        assert_eq!(rcs.some_rust_data, 123u64)
    })
    .join()
    .unwrap();
}

#[test]
fn test_addr() {
    let test_addr: *const usize = 5158485 as _;
    let sync = test_addr.as_sync_const();
    assert_eq!(sync.as_address(), test_addr as usize);
}

#[inline(always)]
extern "C" fn test_function() -> u64 {
    123456u64
}

#[inline(always)]
#[cfg(feature = "fnptr")]
extern "C" fn test_function2() -> u64 {
    1234567u64
}
#[test]
fn test_function_pointer() {
    let test_fn_ptr: SyncMutPtr<c_void> = SyncMutPtr::new(test_function as *mut c_void);
    let result =
        unsafe { mem::transmute::<*mut c_void, extern "C" fn() -> u64>(test_fn_ptr.inner()) }();

    assert_eq!(result, test_function());
}

#[test]
#[cfg(feature = "fnptr")]
fn test_function_pointer2() {
    let test_fn_ptr = sync_fn_ptr!(extern "C" fn() -> u64, test_function);
    assert!(!test_fn_ptr.is_null());
    assert_eq!(test_fn_ptr(), test_function());
}

#[test]
#[cfg(feature = "fnptr")]
fn test_function_pointer3() {
    let n = test_function as *const c_void;

    let test_fn_ptr = unsafe { sync_fn_ptr_from_addr!(extern "C" fn() -> u64, n) };

    assert!(!test_fn_ptr.is_null());
    assert_eq!(test_fn_ptr(), test_function());
}

#[test]
#[cfg(feature = "fnptr")]
fn test_function_pointer_null() {
    let test_fn_ptr = unsafe { sync_fn_ptr_from_addr!(extern "C" fn() -> u64, 0) };

    assert!(test_fn_ptr.is_null());

    let r = std::panic::catch_unwind(move || {
        test_fn_ptr();
    });

    assert!(r.is_err());

    let test_fn_ptr2 = SyncFnPtr::<extern "C" fn() -> u64>::default();

    let r = std::panic::catch_unwind(move || {
        test_fn_ptr2();
    });

    assert!(r.is_err());

    assert_eq!(test_fn_ptr, test_fn_ptr2);
}

#[test]
#[cfg(feature = "fnptr")]
fn test_function_pointer_equal_sync() {
    let n = test_function as *const c_void;
    let test_fn_ptr = unsafe { sync_fn_ptr_from_addr!(extern "C" fn() -> u64, n) };
    let test_fn_ptr2 = sync_fn_ptr!(extern "C" fn() -> u64, test_function);

    //Compiler doesn't guarantee this...
    //assert_eq!(test_fn_ptr, test_fn_ptr2);
    //assert_eq!(test_fn_ptr.as_address(), test_fn_ptr2.as_address());

    if test_fn_ptr == test_fn_ptr2
        || test_fn_ptr.as_address() == test_fn_ptr2.as_address()
        || n as usize == test_function as usize
    {
        assert_eq!(test_fn_ptr.as_address(), test_fn_ptr2.as_address());
        assert_eq!(test_fn_ptr, test_fn_ptr2);
    }

    let test_fn_ptr3 = sync_fn_ptr!(extern "C" fn() -> u64, test_function2);
    assert_ne!(test_fn_ptr, test_fn_ptr3);
    assert_ne!(test_fn_ptr.as_address(), test_fn_ptr3.as_address());
}

#[test]
#[cfg(feature = "fnptr")]
fn test_function_pointer_equal_send() {
    let n = test_function as *const c_void;
    let test_fn_ptr = unsafe { send_fn_ptr_from_addr!(extern "C" fn() -> u64, n) };
    let test_fn_ptr2 = send_fn_ptr!(extern "C" fn() -> u64, test_function);

    //Compiler doesn't guarantee this...
    //assert_eq!(test_fn_ptr, test_fn_ptr2);
    //assert_eq!(test_fn_ptr.as_address(), test_fn_ptr2.as_address());

    if test_fn_ptr == test_fn_ptr2
        || test_fn_ptr.as_address() == test_fn_ptr2.as_address()
        || n as usize == test_function as usize
    {
        assert_eq!(test_fn_ptr.as_address(), test_fn_ptr2.as_address());
        assert_eq!(test_fn_ptr, test_fn_ptr2);
    }

    let test_fn_ptr3 = send_fn_ptr!(extern "C" fn() -> u64, test_function2);
    assert_ne!(test_fn_ptr, test_fn_ptr3);
    assert_ne!(test_fn_ptr.as_address(), test_fn_ptr3.as_address());
}

#[test]
fn snd_test_function_pointer() {
    let test_fn_ptr: SendMutPtr<c_void> = SendMutPtr::new(test_function as *mut c_void);
    let result =
        unsafe { mem::transmute::<*mut c_void, extern "C" fn() -> u64>(test_fn_ptr.inner()) }();

    assert_eq!(result, test_function());
}

#[test]
#[cfg(feature = "fnptr")]
fn snd_test_function_pointer2() {
    let test_fn_ptr = send_fn_ptr!(extern "C" fn() -> u64, test_function);
    assert!(!test_fn_ptr.is_null());
    assert_eq!(test_fn_ptr(), test_function());
}

#[test]
#[cfg(feature = "fnptr")]
fn snd_test_function_pointer3() {
    let n = test_function as *const c_void;

    let test_fn_ptr = unsafe { send_fn_ptr_from_addr!(extern "C" fn() -> u64, n) };

    assert!(!test_fn_ptr.is_null());
    assert_eq!(test_fn_ptr(), test_function());
}

#[test]
#[cfg(feature = "fnptr")]
fn snd_test_function_pointer_null() {
    let test_fn_ptr = unsafe { send_fn_ptr_from_addr!(extern "C" fn() -> u64, 0) };

    assert!(test_fn_ptr.is_null());
    assert!(test_fn_ptr.is_null());
    assert_eq!(test_fn_ptr.as_address(), 0);
    assert_eq!(test_fn_ptr.as_raw_ptr(), std::ptr::null());
    assert!(test_fn_ptr.inner().is_none());

    let r = std::panic::catch_unwind(move || {
        test_fn_ptr();
    });

    assert!(r.is_err());

    let test_fn_ptr2 = SendFnPtr::<extern "C" fn() -> u64>::default();

    assert!(test_fn_ptr2.is_null());
    assert_eq!(test_fn_ptr2.as_address(), 0);
    assert_eq!(test_fn_ptr2.as_raw_ptr(), std::ptr::null());
    assert!(test_fn_ptr2.inner().is_none());
    let r = std::panic::catch_unwind(move || {
        test_fn_ptr2();
    });

    assert!(r.is_err());

    assert_eq!(test_fn_ptr, test_fn_ptr2);

    let test_fn_ptr3 = SendFnPtr::<extern "C" fn() -> u64>::null();
    assert!(test_fn_ptr3.is_null());
    assert_eq!(test_fn_ptr3.as_address(), 0);
    assert_eq!(test_fn_ptr3.as_raw_ptr(), std::ptr::null());
    assert!(test_fn_ptr3.inner().is_none());

    let r = std::panic::catch_unwind(move || {
        test_fn_ptr3();
    });

    assert!(r.is_err());

    assert_eq!(test_fn_ptr, test_fn_ptr3);
}

#[test]
#[cfg(feature = "fnptr")]
fn snd_test_function_pointer_equal() {
    let n = test_function as *const c_void;
    let test_fn_ptr = unsafe { send_fn_ptr_from_addr!(extern "C" fn() -> u64, n) };
    let test_fn_ptr2 = send_fn_ptr!(extern "C" fn() -> u64, test_function);

    //Compiler doesn't guarantee this...
    //assert_eq!(test_fn_ptr, test_fn_ptr2);
    //assert_eq!(test_fn_ptr.as_address(), test_fn_ptr2.as_address());

    if test_fn_ptr == test_fn_ptr2
        || test_fn_ptr.as_address() == test_fn_ptr2.as_address()
        || n as usize == test_function as usize
    {
        //But this is guaranteed
        assert_eq!(test_fn_ptr, test_fn_ptr2);
        assert_eq!(test_fn_ptr.as_address(), test_fn_ptr2.as_address());
    }

    let test_fn_ptr3 = send_fn_ptr!(extern "C" fn() -> u64, test_function2);
    assert_ne!(test_fn_ptr, test_fn_ptr3);
    assert_ne!(test_fn_ptr.as_address(), test_fn_ptr3.as_address());
}

#[test]
fn test_fmt() {
    unsafe {
        let mut memory = vec![0u8; 12345];
        let handle: *mut c_void = memory.as_mut_ptr().add(12345).cast();
        assert_eq!(
            format!("{:p}", handle.as_send_const()).as_str(),
            format!("{:p}", handle).as_str()
        );
        assert_eq!(
            format!("{:p}", handle.as_send_mut()).as_str(),
            format!("{:p}", handle).as_str()
        );
        assert_eq!(
            format!("{:p}", handle.as_sync_mut()).as_str(),
            format!("{:p}", handle).as_str()
        );
        assert_eq!(
            format!("{:p}", handle.as_sync_const()).as_str(),
            format!("{:p}", handle).as_str()
        );
        assert_ne!(
            format!("{:p}", handle.as_sync_const()).as_str(),
            format!("{:p}", memory.as_ptr()).as_str()
        );
    }
}

#[cfg(target_has_atomic = "32")]
#[test]
fn test_mt() {
    unsafe {
        let n = Box::into_raw(Box::new(123));
        let ptr = n.as_sync_mut();
        let jh = std::thread::spawn(move || {
            assert_eq!(123, ptr.read_volatile());
            ptr.write_volatile(456);
        });
        jh.join().unwrap();
        assert_eq!(n.read_volatile(), 456);
        _ = Box::from_raw(n);
    }
}

#[test]
#[cfg(feature = "fnptr")]
fn test_fn_ptr_opt() {
    let ffi_fn: Option<extern "C" fn() -> u64> = Some(test_function);
    let sync = sync_fn_ptr_opt!(extern "C" fn() -> u64, ffi_fn);
    let send = send_fn_ptr_opt!(extern "C" fn() -> u64, ffi_fn);
    let jh = std::thread::spawn(move || sync());
    assert_eq!(123456u64, jh.join().unwrap());

    let jh = std::thread::spawn(move || send());
    assert_eq!(123456u64, jh.join().unwrap());
}
