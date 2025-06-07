// SPDX-License-Identifier: GPL-2.0

//! Compile-time asserts.

/// Asserts that the given type is [`Sync`].
///
/// This check is done at compile time and does nothing at runtime.
///
/// # Examples
///
/// ```
/// # use kernel::compile_assert::assert_sync;
/// // Succeeds because `i32` is `Sync`
/// assert_sync::<i32>();
/// ```
///
/// ```ignore # // TODO: This should be compile_fail once that works
/// # use kernel::compile_assert::assert_sync;
/// # use core::cell::UnsafeCell;
/// // Fails because `NotThreadSafe` is not `Sync`.
/// assert_sync::<UnsafeCell<()>>();
/// ```
#[inline(always)]
pub const fn assert_sync<T: ?Sized + Sync>() {}

/// Asserts that the given type is [`Send`].
///
/// This check is done at compile time and does nothing at runtime.
///
/// # Examples
///
/// ```
/// # use kernel::compile_assert::assert_send;
/// # use core::cell::UnsafeCell;
/// // Succeeds because `i32` is `Send`
/// assert_send::<i32>();
/// // Succeeds because `UnsafeCell<T>` is `Send` if `T: Send`
/// assert_send::<UnsafeCell<()>>();
/// ```
///
/// ```ignore # // TODO: This should be compile_fail once that works
/// # use kernel::compile_assert::assert_send;
/// # use kernel::NotThreadSafe;
/// // Fails because `NotThreadSafe` is not `Send`.
/// assert_send::<NotThreadSafe>();
/// ```
#[inline(always)]
pub const fn assert_send<T: ?Sized + Send>() {}
