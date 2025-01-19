// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The contents of this file partially come from the Rust standard library, hosted in
//! the <https://github.com/rust-lang/rust> repository, licensed under
//! "Apache-2.0 OR MIT" and adapted for kernel use. For copyright details,
//! see <https://github.com/rust-lang/rust/blob/master/COPYRIGHT>.
//!
//! This file provides a implementation / polyfill of a subset of the upstream
//! rust `UnsafePinned` type. For details on the difference to the upstream
//! implementation see the comment on the [`UnsafePinned`] struct definition.

use core::{cell::UnsafeCell, marker::PhantomPinned};
use pin_init::{cast_pin_init, PinInit, Wrapper};

/// This type provides a way to entirely opt-out of typical aliasing rules;
/// specifically, `&mut UnsafePinned<T>` is not guaranteed to be a unique pointer.
/// This also subsumes the effects of `UnsafeCell`, i.e., `&UnsafePinned<T>` may point to data
/// that is being mutated.
///
/// However, even if you define your type like `pub struct Wrapper(UnsafePinned<...>)`, it is still
/// very risky to have an `&mut Wrapper` that aliases anything else. Many functions that work
/// generically on `&mut T` assume that the memory that stores `T` is uniquely owned (such as
/// `mem::swap`). In other words, while having aliasing with `&mut Wrapper` is not immediate
/// Undefined Behavior, it is still unsound to expose such a mutable reference to code you do not
/// control! Techniques such as pinning via [`Pin`](core::pin::Pin) are needed to ensure soundness.
///
/// Similar to [`UnsafeCell`], [`UnsafePinned`] will not usually show up in
/// the public API of a library. It is an internal implementation detail of libraries that need to
/// support aliasing mutable references.
///
/// This type blocks niches the same way [`UnsafeCell`] does.
///
/// # Kernel implementation notes
///
/// This implementation works because of the "`!Unpin` hack" in rustc, which allows (some kinds of)
/// mutual aliasing of `!Unpin` types. This hack might be removed at some point, after which only
/// the `core::pin::UnsafePinned` type will allow this behavior. In order to simplify the migration
/// to future rust versions only this polyfill of this type should be used when this behavior is
/// required.
//
// As opposed to the upstream Rust type this contains a `PhantomPinned` to ensure the struct always
// is `!Unpin` the upstream type has a explicit impl `impl !Unpin for UnsafePinned`.
#[repr(transparent)]
pub struct UnsafePinned<T: ?Sized> {
    _ph: PhantomPinned,
    value: UnsafeCell<T>,
}

impl<T> UnsafePinned<T> {
    /// Constructs a new instance of [`UnsafePinned`] which will wrap the specified value.
    ///
    /// All access to the inner value through `&UnsafePinned<T>` or `&mut UnsafePinned<T>` or
    /// `Pin<&mut UnsafePinned<T>>` requires `unsafe` code.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: T) -> Self {
        UnsafePinned {
            value: UnsafeCell::new(value),
            _ph: PhantomPinned,
        }
    }
}
impl<T: ?Sized> UnsafePinned<T> {
    /// Get mutable access to the contents of a shared `UnsafePinned`.
    ///
    /// This can be cast to a pointer of any kind. When creating references, you must uphold the
    /// aliasing rules; see [`UnsafeCell`] for more discussion and caveats.
    ///
    /// [`UnsafeCell`]: core::cell::UnsafeCell#aliasing-rules
    ///
    /// ```rust,no_run
    /// use kernel::types::UnsafePinned;
    ///
    /// unsafe {
    ///     let mut x = UnsafePinned::new(0);
    ///     let ptr = x.get();
    ///     UnsafePinned::raw_get(&raw mut x).write(1);
    ///     assert_eq!(ptr.read(), 1);
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn get(&self) -> *mut T {
        self.value.get()
    }

    /// Gets a mutable pointer to the wrapped value.
    #[inline(always)]
    #[must_use]
    pub const fn raw_get(this: *mut Self) -> *mut T {
        this as *mut T
    }
}

impl<T> Wrapper<T> for UnsafePinned<T> {
    fn pin_init<E>(init: impl PinInit<T, E>) -> impl PinInit<Self, E> {
        // SAFETY: `UnsafePinned<T>` has a compatible layout to `T`.
        unsafe { cast_pin_init(init) }
    }
}
