# sync-ptr
Sync & Send wrappers for raw pointer's and function pointers in rust.

This crate uses `#![no_std]` and can be used in projects that do not use the rust standard library.

Minimum rust version is rust 1.65.0

### Intended use
This crate is intended for handles or pointers to data behind an FFI boundary 
that is known to be Send or Sync. 

Example where this is most likely the case:
- shared memory from mmap() or MapViewOfFile()
- global JNI handles
- ffi mutex handles etc
- ffi function pointers
- ...

I would like to mention that this crate does not magically make any 
C/FFI pointers into arbitrary data Send or Sync safely.
Using pointers/handles from a different thread can be UB in rust, 
or you may cause UB on the C/FFI side.

This crate is not intended for usage with pointers to rust data.
While it can be used like this, one has to be VERY careful to avoid UB.

### Example

```rust
use std::ffi::c_void;
use std::ptr::null_mut;
use sync_ptr::*;

struct RustControlStructureThatIsNowSend {
    some_handle: SendConstPtr<c_void>,
    some_rust_data: u64
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
```

#### Why not just make RustControlStructureThatIsNowSend implement Send directly?
This is prone to error as once a struct is "unsafe impl Sync," for example, it will be Sync no matter what
struct members get added later. If the initial reason for that unsafe impl was a raw pointer, then 
the compiler has no opportunity to inform the Human that adding a RefCell to such a struct is maybe not a good idea.

In addition, there are sometimes cases where one only needs to 
send a single pointer, and writing an unsafe impl wrapper struct
everytime is annoying.

### Function pointers
While you can use the `SyncConstPtr` for example to wrap your raw function pointers, by first casting them to a pointer type like c_void,
doing this is cumbersome as it requires you to transmute the pointer back to a function pointer every time you want to use it.

To make this easier this crate provides a specialized wrapper for function pointers 
that allows you to call the function without unsafe code or specifying the signature every time.
The two wrappers are called `SyncFnPtr` and `SendFnPtr`.

These wrappers are not provided by default because they rely on rust features that were only stabilized in rust 1.85.0.
If your msrv is 1.85.0 or newer, then use the "fnptr" feature to enable this.

#### Null Function pointers
Rust does not support null function pointers, 
but since null function pointers are very common in FFI this crate does support them.
`SyncFnPtr` and `SendFnPtr` both impl `Default` as well as provide a `null` constructor, 
which provides such a null function pointer. If you attempt to call a null function pointer 
then your program is guaranteed to panic.

Internally both wrappers use repr(transparent) of `Option<fn(...) -> ...>` granting it the same layout.
This makes the wrappers suitable for use in FFI structs.

#### Constructing the function pointer wrapper
Unfortunately the trait `core::marker::FnPtr` is unstable, this prevents this crate from providing
a convenient safe constructor function for the wrappers.

The only way to construct the wrappers (other than using null/default) is using one of the 4 provided macros:
- `send_fn_ptr!` - For use with rust functions only
- `sync_fn_ptr!` - For use with rust functions only
- `send_fn_ptr_from_addr!` - For use with usize/raw pointer
- `sync_fn_ptr_from_addr!` - For use with usize/raw pointer

#### Example
Constructing these wrappers from rust functions does not require unsafe code, 
but constructing them from raw pointers or usize does need unsafe code.

```rust
use std::ffi::c_void;
use sync_ptr::send_fn_ptr;
use sync_ptr::send_fn_ptr_from_addr;
use sync_ptr::SendFnPtr;

extern "C" fn test_function() -> u64 {
    123456u64
}

fn some_function() {
    //Constructing from a rust function
    //First argument to the marco is the function signature. 
    //Second argument ref rust function.
    //The macro will check that the provided function matches the signature. If not it will cause a compiler error.
    //It's not possible to make mistakes with the signature here.
    let test_fn_ptr = send_fn_ptr!(extern "C" fn() -> u64, test_function);
    
    //Type of test_fn_ptr is SendFnPtr<extern "C" fn() -> u64>
    std::thread::spawn(move || {
        assert_eq!(test_fn_ptr(), test_function());
    }).join().unwrap();
    
    //Constructing from an address:
    //usually you would get this address from ffi/dlsym/GetProcAddress.
    let some_address = test_function as *const c_void;
    //First argument is once again the function signature. 
    //Second argument can be:
    // - 1. a rust function (why, use the safe macro above)
    // - 2. usize
    // - 3. any *mut/*const raw pointer. (I recommend using c_void, but it will compile with any raw pointer)
    // Unlike the safe macro above, this macro has no way to check if the function signature is correct.
    // If the address/pointer is not valid then calling this macro results in UB.
    // Pass a null pointer or 0usize is not UB and will yield the same result as the SendFnPtr::default/null fn's.
    let test_fn_ptr = unsafe { send_fn_ptr_from_addr!(extern "C" fn() -> u64, some_address) };
    //Type of test_fn_ptr is SendFnPtr<extern "C" fn() -> u64>
    std::thread::spawn(move || {
        assert_eq!(test_fn_ptr(), test_function());
    }).join().unwrap();
}
```
