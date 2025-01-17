// SPDX-License-Identifier: GPL-2.0

//! Rust Platform driver for pushbutton

use kernel::{c_str, devres::Devres, io::mem::IoMem, of, platform, prelude::*};

kernel::of_device_table!(
    OF_TABLE,
    MODULE_OF_TABLE,
    <PushbuttonDriver as platform::Driver>::IdInfo,
    [(of::DeviceId::new(c_str!("ldd,pushbutton")), ())]
);

const MAPPING_SIZE: usize = 0x10;
const INTERRUPT_MASK_OFFSET: usize = 0x8;
const EDGE_CAPTURE_OFFSET: usize = 0xC;

const BUTTON_MASK: u32 = 0b1111;

#[pin_data(PinnedDrop)]
struct PushbuttonDriver {
    pdev: platform::Device,
    mapping: Devres<IoMem<MAPPING_SIZE, true>>,
}

impl platform::Driver for PushbuttonDriver {
    type IdInfo = ();
    const OF_ID_TABLE: Option<of::IdTable<Self::IdInfo>> = Some(&OF_TABLE);

    fn probe(pdev: &mut platform::Device, info: Option<&Self::IdInfo>) -> Result<Pin<KBox<Self>>> {
        dev_dbg!(pdev.as_ref(), "Probe Rust pushbutton driver.\n");

        if info.is_some() {
            dev_info!(pdev.as_ref(), "Probed with info\n");
        }

        let mapping = pdev
            .ioremap_resource_sized::<MAPPING_SIZE, true>(pdev.resource(0).ok_or(ENXIO)?)
            .map_err(|_| ENXIO)?;

        let res = mapping.try_access().ok_or(ENXIO)?;
        res.writel(BUTTON_MASK, INTERRUPT_MASK_OFFSET);
        res.writel(BUTTON_MASK, EDGE_CAPTURE_OFFSET);

        let drvdata = KBox::try_pin_init(
            try_pin_init!(Self {
                pdev: pdev.clone(),
                mapping,
            }),
            GFP_KERNEL,
        )?;

        Ok(drvdata)
    }
}

#[pinned_drop]
impl PinnedDrop for PushbuttonDriver {
    fn drop(self: Pin<&mut Self>) {
        dev_info!(self.pdev.as_ref(), "Remove Rust pushbutton driver.\n");

        if let Some(res) = self.mapping.try_access() {
            res.writel(0, INTERRUPT_MASK_OFFSET);
            res.writel(0, EDGE_CAPTURE_OFFSET);
        }
    }
}

kernel::module_platform_driver! {
    type: PushbuttonDriver,
    name: "pushbutton",
    author: "Christian Schrefl",
    description: "Rust pushbutton driver",
    license: "GPL v2",
}
