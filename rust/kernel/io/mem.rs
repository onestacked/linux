// SPDX-License-Identifier: GPL-2.0

//! Generic memory-mapped IO.

use core::ops::Deref;

use crate::device::Device;
use crate::devres::Devres;
use crate::io::resource::Resource;
use crate::io::Io;
use crate::io::IoRaw;
use crate::prelude::*;

/// A generic memory-mapped IO region.
///
/// Accesses to the underlying region is checked either at compile time, if the
/// region's size is known at that point, or at runtime otherwise.
///
/// Whether `IoMem` represents an exclusive access to the underlying memory
/// region is determined by the caller at creation time, as overlapping access
/// may be needed in some cases.
///
/// # Invariants
///
/// `IoMem` always holds an `IoRaw` inststance that holds a valid pointer to the
/// start of the I/O memory mapped region.
pub struct IoMem<const SIZE: usize = 0, const EXCLUSIVE: bool = true> {
    io: IoRaw<SIZE>,
    res_start: u64,
}

impl<const SIZE: usize, const EXCLUSIVE: bool> IoMem<SIZE, EXCLUSIVE> {
    /// Creates a new `IoMem` instance.
    pub(crate) fn new(resource: &Resource, device: &Device) -> Result<Devres<Self>> {
        let size = resource.size();
        if size == 0 {
            return Err(EINVAL);
        }

        let res_start = resource.start();

        if EXCLUSIVE {
            // SAFETY:
            // - `res_start` and `size` are read from a presumably valid `struct resource`.
            // - `size` is known not to be zero at this point.
            // - `resource.name()` returns a valid C string.
            let mem_region = unsafe {
                bindings::request_mem_region(res_start, size, resource.name().as_char_ptr())
            };

            if mem_region.is_null() {
                return Err(EBUSY);
            }
        }

        // SAFETY:
        // - `res_start` and `size` are read from a presumably valid `struct resource`.
        // - `size` is known not to be zero at this point.
        let addr = unsafe { bindings::ioremap(res_start, size as kernel::ffi::c_ulong) };
        if addr.is_null() {
            if EXCLUSIVE {
                // SAFETY:
                // - `res_start` and `size` are read from a presumably valid `struct resource`.
                // - `size` is the same as the one passed to `request_mem_region`.
                unsafe { bindings::release_mem_region(res_start, size) };
            }
            return Err(ENOMEM);
        }

        let io = IoRaw::new(addr as usize, size as usize)?;
        let io = IoMem { io, res_start };
        let devres = Devres::new(device, io, GFP_KERNEL)?;

        Ok(devres)
    }
}

impl<const SIZE: usize, const EXCLUSIVE: bool> Drop for IoMem<SIZE, EXCLUSIVE> {
    fn drop(&mut self) {
        if EXCLUSIVE {
            // SAFETY: `res_start` and `io.maxsize()` were the values passed to
            // `request_mem_region`.
            unsafe { bindings::release_mem_region(self.res_start, self.io.maxsize() as u64) }
        }

        // SAFETY: Safe as by the invariant of `Io`.
        unsafe { bindings::iounmap(self.io.addr() as *mut core::ffi::c_void) }
    }
}

impl<const SIZE: usize, const EXCLUSIVE: bool> Deref for IoMem<SIZE, EXCLUSIVE> {
    type Target = Io<SIZE>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Safe as by the invariant of `IoMem`.
        unsafe { Io::from_raw(&self.io) }
    }
}
