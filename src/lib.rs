//! # sync-ptr
//! Sync & Send wrappers for raw pointer's in rust.
//! To use add `use sync_ptr::*;` to your file,
//! then you should be able to call `my_ptr.as_sync_const()` among others on any raw pointer
//! to get a wrapped version of your raw pointer that is Sync/Send.
//!
//! Example:
//! ```rust
//! use std::ffi::c_void;
//! use sync_ptr::*;
//!
//! fn my_func(some_ptr: *mut c_void) {
//!     let ptr: SyncMutPtr<c_void> = some_ptr.as_sync_mut();
//!     std::thread::spawn(move || {
//!         let _some_ptr : *mut c_void = ptr.inner();
//!     });
//! }
//!
//! ```
//!
#![no_std]
#![deny(clippy::correctness)]
#![warn(
    clippy::perf,
    clippy::complexity,
    clippy::style,
    clippy::nursery,
    clippy::pedantic,
    clippy::clone_on_ref_ptr,
    clippy::decimal_literal_representation,
    clippy::float_cmp_const,
    clippy::missing_docs_in_private_items,
    clippy::multiple_inherent_impl,
    clippy::unwrap_used,
    clippy::cargo_common_metadata,
    clippy::used_underscore_binding
)]
#![allow(clippy::inline_always)]

use core::fmt::{Formatter, Pointer};
use core::ops::Deref;

/// Implement common traits for type `SelfType` by forwarding implementation
/// to an underlying pointer.
///
/// Rust compiler cannot correctly auto-derive them because it's adding unnecessary
/// constraint equivalent to:
///
/// ```ignore
/// impl<T: Clone> Clone for SyncMutPtr<T> {...}
/// ```
///
/// It's not consistent with how these traits are implemented in built-in primitive pointers:
/// for example, a pointer can be cloned even if the underlying type does not implement Clone, because
/// we are cloning a pointer, not the value it points to.
///
/// To make the implementation of traits in this library consistent with the implementation of same
/// traits on primitive pointers, we have to manually implement them.
macro_rules! trait_impl {
    ($SelfType:ident) => {
        impl<T> Clone for $SelfType<T> {
            #[inline(always)]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<T> Copy for $SelfType<T> {}
        impl<T> Pointer for $SelfType<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                core::fmt::Pointer::fmt(&self.0, f)
            }
        }

        impl<T> Eq for $SelfType<T> {}
        impl<T> PartialEq for $SelfType<T> {
            #[inline(always)]
            fn eq(&self, other: &Self) -> bool {
                PartialEq::eq(&self.0, &other.0)
            }
        }

        impl<T> PartialOrd for $SelfType<T> {
            #[inline(always)]
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<T> Ord for $SelfType<T> {
            #[inline(always)]
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                Ord::cmp(&self.0, &other.0)
            }
        }

        impl<T> core::fmt::Debug for $SelfType<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple(stringify!($SelfType)).field(&self.0).finish()
            }
        }

        impl<T> core::hash::Hash for $SelfType<T> {
            #[inline(always)]
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                core::hash::Hash::hash(&self.0, state);
            }
        }

        impl<T> From<$SelfType<T>> for usize {
            #[inline(always)]
            fn from(val: $SelfType<T>) -> Self {
                val.as_address()
            }
        }

        impl<T> From<usize> for $SelfType<T> {
            #[inline(always)]
            fn from(value: usize) -> Self {
                Self::from_address(value)
            }
        }
    };
}

///
/// Wrapped mutable raw pointer that is Send+Sync
///
#[repr(transparent)]
pub struct SyncMutPtr<T>(*mut T);

unsafe impl<T> Sync for SyncMutPtr<T> {}
unsafe impl<T> Send for SyncMutPtr<T> {}

trait_impl!(SyncMutPtr);
impl<T> Deref for SyncMutPtr<T> {
    type Target = *mut T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<*mut T> for SyncMutPtr<T> {
    #[inline(always)]
    fn from(value: *mut T) -> Self {
        Self(value)
    }
}

impl<T> From<SyncMutPtr<T>> for *mut T {
    #[inline(always)]
    fn from(val: SyncMutPtr<T>) -> Self {
        val.inner()
    }
}

impl<T> From<SyncMutPtr<T>> for *const T {
    #[inline(always)]
    fn from(val: SyncMutPtr<T>) -> Self {
        val.inner()
    }
}

impl<T> SyncMutPtr<T> {
    ///
    /// Makes `ptr` Send+Sync
    ///
    /// # Safety
    /// The `ptr` parameter must be able to handle being sent and used in other threads concurrently,
    /// or special care must be taken when using the wrapped `ptr` to not use it
    /// in any way in other threads.
    ///
    /// Note: This function is potentially always safe to call, rusts specification does not really make
    /// it clear why
    ///
    #[inline(always)]
    #[must_use]
    pub const fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }

    ///
    /// Returns a `SyncMutPtr` from an arbitrary address.
    /// This is equivalent to casting `usize as *mut T`
    ///
    #[inline(always)]
    #[must_use]
    pub const fn from_address(addr: usize) -> Self {
        Self(addr as *mut T)
    }

    ///
    /// Returns the address of the pointer.
    /// This is equivalent to casting the pointer using `*mut T as usize`.
    ///
    /// # Note
    /// Starting with rust `1.84.0`, the pointer itself
    /// has the functions `addr` and `expose_provenance`.
    /// These functions should be used instead.
    /// They are available via the deref trait.
    /// This function is roughly equivalent to the `expose_provenance` function.
    ///
    #[inline(always)]
    #[must_use]
    pub fn as_address(&self) -> usize {
        self.0 as usize
    }

    ///
    /// Makes a Send+Sync null ptr.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn null() -> Self {
        Self(core::ptr::null_mut())
    }

    ///
    /// Casts `ptr` to another data type while keeping it Send+Sync.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn cast<Y>(&self) -> SyncMutPtr<Y> {
        SyncMutPtr(self.0.cast())
    }

    ///
    /// Returns inner `ptr`, which is then no longer Send+Sync.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn inner(&self) -> *mut T {
        self.0
    }

    ///
    /// Makes `ptr` immutable.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_sync_const(&self) -> SyncConstPtr<T> {
        SyncConstPtr(self.0)
    }

    ///
    /// Makes `ptr` immutable and no longer Sync.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_send_const(&self) -> SendConstPtr<T> {
        SendConstPtr(self.0)
    }

    ///
    /// This is equivalent to `.clone()` and does nothing.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_sync_mut(&self) -> Self {
        Self(self.0)
    }

    ///
    /// Makes `ptr` no longer Sync.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_send_mut(&self) -> SendMutPtr<T> {
        SendMutPtr(self.0)
    }
}

///
/// Wrapped const raw pointer that is Send+Sync
///
#[repr(transparent)]
pub struct SyncConstPtr<T>(*const T);

unsafe impl<T> Sync for SyncConstPtr<T> {}
unsafe impl<T> Send for SyncConstPtr<T> {}

trait_impl!(SyncConstPtr);

impl<T> Deref for SyncConstPtr<T> {
    type Target = *const T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<*mut T> for SyncConstPtr<T> {
    #[inline(always)]
    fn from(value: *mut T) -> Self {
        Self(value)
    }
}

impl<T> From<*const T> for SyncConstPtr<T> {
    #[inline(always)]
    fn from(value: *const T) -> Self {
        Self(value)
    }
}

impl<T> From<SyncConstPtr<T>> for *const T {
    #[inline(always)]
    fn from(val: SyncConstPtr<T>) -> Self {
        val.inner()
    }
}

impl<T> SyncConstPtr<T> {
    ///
    /// Makes `ptr` Send+Sync
    ///
    #[inline(always)]
    #[must_use]
    pub const fn new(ptr: *const T) -> Self {
        Self(ptr)
    }

    ///
    /// Returns a `SyncConstPtr` from an arbitrary address.
    /// This is equivalent to casting `usize as *const T`
    ///
    #[inline(always)]
    #[must_use]
    pub const fn from_address(addr: usize) -> Self {
        Self(addr as *mut T)
    }

    ///
    /// Returns the address of the pointer.
    /// This is equivalent to casting the pointer using `*mut T as usize`.
    ///
    /// # Note
    /// Starting with rust `1.84.0`, the pointer itself
    /// has the functions `addr` and `expose_provenance`.
    /// These functions should be used instead.
    /// They are available via the deref trait.
    /// This function is roughly equivalent to the `expose_provenance` function.
    ///
    #[inline(always)]
    #[must_use]
    pub fn as_address(&self) -> usize {
        self.0 as usize
    }

    ///
    /// Makes a Send+Sync null ptr.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn null() -> Self {
        Self(core::ptr::null())
    }

    ///
    /// Casts `ptr` to another data type while keeping it Send+Sync.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn cast<Y>(&self) -> SyncConstPtr<Y> {
        SyncConstPtr(self.0.cast())
    }

    ///
    /// Returns inner `ptr`, which is then no longer Send+Sync.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn inner(&self) -> *const T {
        self.0
    }

    ///
    /// This is equivalent to `.clone()` and does nothing.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_sync_const(&self) -> Self {
        Self(self.0)
    }

    ///
    /// Makes this `ptr` no longer Sync.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_send_const(&self) -> SendConstPtr<T> {
        SendConstPtr(self.0)
    }

    ///
    /// Makes this `ptr` mutable
    ///
    /// # Safety
    /// Writing to immutable data is UB.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_sync_mut(&self) -> SyncMutPtr<T> {
        SyncMutPtr(self.0.cast_mut())
    }

    ///
    /// Makes this `ptr` mutable and no longer Sync.
    ///
    /// # Safety
    /// Writing to immutable data is UB.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_send_mut(&self) -> SendMutPtr<T> {
        SendMutPtr(self.0.cast_mut())
    }
}

///
/// Wrapped mutable raw pointer that is Send but not Sync
///
#[repr(transparent)]
pub struct SendMutPtr<T>(*mut T);

unsafe impl<T> Send for SendMutPtr<T> {}

trait_impl!(SendMutPtr);

impl<T> Deref for SendMutPtr<T> {
    type Target = *mut T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<SendMutPtr<T>> for *mut T {
    #[inline(always)]
    fn from(val: SendMutPtr<T>) -> Self {
        val.inner()
    }
}

impl<T> From<SendMutPtr<T>> for *const T {
    #[inline(always)]
    fn from(val: SendMutPtr<T>) -> Self {
        val.inner()
    }
}

impl<T> SendMutPtr<T> {
    ///
    /// Makes `ptr` Send
    ///
    #[inline(always)]
    #[must_use]
    pub const fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }

    ///
    /// Returns a `SendMutPtr` from an arbitrary address.
    /// This is equivalent to casting `usize as *mut T`
    ///
    #[inline(always)]
    #[must_use]
    pub const fn from_address(addr: usize) -> Self {
        Self(addr as *mut T)
    }

    ///
    /// Returns the address of the pointer.
    /// This is equivalent to casting the pointer using `*mut T as usize`.
    ///
    /// # Note
    /// Starting with rust `1.84.0`, the pointer itself
    /// has the functions `addr` and `expose_provenance`.
    /// These functions should be used instead.
    /// They are available via the deref trait.
    /// This function is roughly equivalent to the `expose_provenance` function.
    ///
    #[inline(always)]
    #[must_use]
    pub fn as_address(&self) -> usize {
        self.0 as usize
    }

    ///
    /// Makes a Send null ptr.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn null() -> Self {
        Self(core::ptr::null_mut())
    }

    ///
    /// Casts `ptr` to another data type while keeping it Send.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn cast<Y>(&self) -> SendMutPtr<Y> {
        SendMutPtr(self.0.cast())
    }

    ///
    /// Returns inner `ptr` which is then no longer Send.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn inner(&self) -> *mut T {
        self.0
    }

    ///
    /// Makes this `ptr` Sync
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_sync_const(&self) -> SyncConstPtr<T> {
        SyncConstPtr(self.0)
    }

    ///
    /// Makes this `ptr` const.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_send_const(&self) -> SendConstPtr<T> {
        SendConstPtr(self.0)
    }

    ///
    /// Makes this `ptr` Sync
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_sync_mut(&self) -> SyncMutPtr<T> {
        SyncMutPtr(self.0)
    }

    ///
    /// This is equivalent to `.clone()` and does nothing.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_send_mut(&self) -> Self {
        Self(self.0)
    }
}

///
/// Wrapped const raw pointer that is Send but not Sync
///
#[repr(transparent)]
pub struct SendConstPtr<T>(*const T);

unsafe impl<T> Send for SendConstPtr<T> {}

trait_impl!(SendConstPtr);

impl<T> Deref for SendConstPtr<T> {
    type Target = *const T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<SendConstPtr<T>> for *const T {
    #[inline(always)]
    fn from(val: SendConstPtr<T>) -> *const T {
        val.inner()
    }
}

impl<T> SendConstPtr<T> {
    ///
    /// Makes `ptr` Send
    ///
    #[inline(always)]
    #[must_use]
    pub const fn new(ptr: *const T) -> Self {
        Self(ptr)
    }

    ///
    /// Returns a `SendConstPtr` from an arbitrary address.
    /// This is equivalent to casting `usize as *const T`
    ///
    #[inline(always)]
    #[must_use]
    pub const fn from_address(addr: usize) -> Self {
        Self(addr as *mut T)
    }

    ///
    /// Returns the address of the pointer.
    /// This is equivalent to casting the pointer using `*mut T as usize`.
    ///
    /// # Note
    /// Starting with rust `1.84.0`, the pointer itself
    /// has the functions `addr` and `expose_provenance`.
    /// These functions should be used instead.
    /// They are available via the deref trait.
    /// This function is roughly equivalent to the `expose_provenance` function.
    ///
    ///
    #[inline(always)]
    #[must_use]
    pub fn as_address(&self) -> usize {
        self.0 as usize
    }

    ///
    /// Makes a Send null ptr.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn null() -> Self {
        Self(core::ptr::null())
    }

    ///
    /// Casts `ptr` to another data type while keeping it Send.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn cast<Y>(&self) -> SendConstPtr<Y> {
        SendConstPtr(self.0.cast())
    }

    ///
    /// Returns inner `ptr` which is then no longer Send.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn inner(&self) -> *const T {
        self.0
    }

    ///
    /// Makes this `ptr` Sync
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_sync_const(&self) -> SyncConstPtr<T> {
        SyncConstPtr(self.0)
    }

    ///
    /// This is equivalent to `.clone()` and does nothing.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_send_const(&self) -> Self {
        Self(self.0)
    }

    ///
    /// Makes this `ptr` Sync
    ///
    /// # Safety
    /// `ptr` is also marked as mutable. Writing to immutable data is usually UB.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_sync_mut(&self) -> SyncMutPtr<T> {
        SyncMutPtr(self.0.cast_mut())
    }

    ///
    /// Makes this `ptr` mutable
    ///
    /// # Safety
    /// Writing to immutable data is UB.
    ///
    #[inline(always)]
    #[must_use]
    pub const fn as_send_mut(&self) -> SendMutPtr<T> {
        SendMutPtr(self.0.cast_mut())
    }
}

/// Helper trait for every `*const T` and `*mut T` to add fn's to wrap it into a Sync/Send wrapper.
///
/// This trait does not need to be implemented directly.
pub trait FromConstPtr<T>: Sized {
    ///
    /// Makes `self` immutable and Send+Sync
    ///
    fn as_sync_const(&self) -> SyncConstPtr<T>;

    ///
    /// Makes `self` immutable and Send
    ///
    fn as_send_const(&self) -> SendConstPtr<T>;
}

/// Helper trait for every `*mut T` to add fn's to wrap it into a Sync/Send wrapper.
///
/// This trait does not need to be implemented directly.
pub trait FromMutPtr<T>: FromConstPtr<T> {
    ///
    /// Makes `self` Send+Sync
    ///
    fn as_sync_mut(&self) -> SyncMutPtr<T>;

    ///
    /// Makes `self` Send
    ///
    fn as_send_mut(&self) -> SendMutPtr<T>;
}

impl<T> FromConstPtr<T> for *const T {
    #[inline(always)]
    fn as_sync_const(&self) -> SyncConstPtr<T> {
        SyncConstPtr(self.cast())
    }

    #[inline(always)]
    fn as_send_const(&self) -> SendConstPtr<T> {
        SendConstPtr(self.cast())
    }
}

impl<T> FromConstPtr<T> for *mut T {
    #[inline(always)]
    fn as_sync_const(&self) -> SyncConstPtr<T> {
        SyncConstPtr(self.cast())
    }

    #[inline(always)]
    fn as_send_const(&self) -> SendConstPtr<T> {
        SendConstPtr(self.cast())
    }
}

impl<T> FromMutPtr<T> for *mut T {
    #[inline(always)]
    fn as_sync_mut(&self) -> SyncMutPtr<T> {
        SyncMutPtr(self.cast())
    }

    #[inline(always)]
    fn as_send_mut(&self) -> SendMutPtr<T> {
        SendMutPtr(self.cast())
    }
}
