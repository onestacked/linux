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
