// SPDX-License-Identifier: GPL-2.0

//! Rust Platform driver for ledpwm

use core::sync::atomic::{AtomicU8, Ordering};
use kernel::{
    c_str, container_of,
    device::Device,
    devres::Devres,
    fs::File,
    io::mem::IoMem,
    miscdevice::{MiscDevice, MiscDeviceOptions, MiscDeviceRegistration},
    of, platform,
    prelude::*,
    types::ARef,
};

kernel::of_device_table!(
    OF_TABLE,
    MODULE_OF_TABLE,
    <LedPwmDriver as platform::Driver>::IdInfo,
    [(of::DeviceId::new(c_str!("ldd,ledpwm")), ())]
);

const MAPPING_SIZE: usize = 0x4;
static NUM: AtomicU8 = AtomicU8::new(0);

#[pin_data(PinnedDrop)]
struct LedPwmDriver {
    pdev: platform::Device,
    mapping: Devres<IoMem<MAPPING_SIZE, true>>,
    #[pin]
    miscdev: MiscDeviceRegistration<RustMiscDevice>,
}

impl platform::Driver for LedPwmDriver {
    type IdInfo = ();
    const OF_ID_TABLE: Option<of::IdTable<Self::IdInfo>> = Some(&OF_TABLE);

    fn probe(pdev: &mut platform::Device, info: Option<&Self::IdInfo>) -> Result<Pin<KBox<Self>>> {
        dev_dbg!(pdev.as_ref(), "Probe Rust Platform driver sample.\n");

        if info.is_some() {
            dev_info!(pdev.as_ref(), "Probed with info\n");
        }

        let mapping = pdev
            .ioremap_resource_sized::<MAPPING_SIZE, true>(pdev.resource(0).ok_or(ENXIO)?)
            .map_err(|_| ENXIO)?;

        // Enable the LEDs on driver load
        mapping.try_access().ok_or(ENXIO)?.writel(0xFF, 0x0);

        let names = [
            c_str!("led0"),
            c_str!("led1"),
            c_str!("led2"),
            c_str!("led3"),
            c_str!("led4"),
            c_str!("led5"),
            c_str!("led6"),
            c_str!("led7"),
            c_str!("led8"),
            c_str!("led9"),
        ];
        let name = names[NUM.fetch_add(1, Ordering::Relaxed) as usize];
        let options = MiscDeviceOptions { name };

        let drvdata = KBox::try_pin_init(
            try_pin_init!(Self {
                pdev: pdev.clone(),
                mapping,
                miscdev <- MiscDeviceRegistration::register(options),
            }),
            GFP_KERNEL,
        )?;

        Ok(drvdata)
    }
}

#[pin_data(PinnedDrop)]
struct RustMiscDevice {
    dev: ARef<Device>,
    mapping: &'static Devres<IoMem<MAPPING_SIZE, true>>,
}

#[vtable]
impl MiscDevice for RustMiscDevice {
    type Ptr = Pin<KBox<Self>>;

    fn open(_file: &File, misc: &MiscDeviceRegistration<Self>) -> Result<Pin<KBox<Self>>> {
        let dev = ARef::from(misc.device());
        // SAFETY:
        // Stuff?
        let ledpwm = unsafe { &*container_of!(misc, LedPwmDriver, miscdev) };

        dev_info!(dev, "Opening Rust Misc Device Sample\n");

        let res = ledpwm.mapping.try_access().ok_or(ENXIO)?;
        res.writel(0x80, 0x0);

        KBox::try_pin_init(
            try_pin_init! {
                RustMiscDevice {
                    dev: dev,
                    mapping: &ledpwm.mapping
                }
            },
            GFP_KERNEL,
        )
    }
}

#[pinned_drop]
impl PinnedDrop for RustMiscDevice {
    fn drop(self: Pin<&mut Self>) {
        dev_info!(self.dev, "Exiting the Rust Misc Device Sample\n");

        if let Some(res) = self.mapping.try_access() {
            res.writel(0x10, 0x0);
        }
    }
}

#[pinned_drop]
impl PinnedDrop for LedPwmDriver {
    fn drop(self: Pin<&mut Self>) {
        dev_info!(self.pdev.as_ref(), "Remove Rust Platform driver sample.\n");

        if let Some(res) = self.mapping.try_access() {
            res.writel(0x00, 0x0);
        }
    }
}

kernel::module_platform_driver! {
    type: LedPwmDriver,
    name: "ledpwm",
    author: "Christian Schrefl",
    description: "Rust Platform driver",
    license: "GPL v2",
}
