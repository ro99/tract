use tract_data::anyhow::{self, bail, Error};

use super::ScratchSpace;
use std::cell::UnsafeCell;

type ScratchLts = UnsafeCell<Option<Box<dyn ScratchSpace>>>;

thread_local!(
    static LOCAL_SCRATCH: ScratchLts = UnsafeCell::new(None);
);

pub fn finish() {
    LOCAL_SCRATCH.with(move |e| unsafe {
        *e.get() = None;
    });
}

fn get_local_scratch() -> &'static mut Option<Box<dyn ScratchSpace>> {
    LOCAL_SCRATCH.with(move |e| unsafe { &mut *e.get() })
}

fn set_local_scratch(scratch: Box<dyn ScratchSpace>) {
    LOCAL_SCRATCH.with(move |e| unsafe {
        *e.get() = Some(scratch);
    });
}

pub fn get_or_init<F>(f: F) -> anyhow::Result<&'static mut Box<dyn ScratchSpace>, Error>
where
    F: FnOnce() -> Box<dyn ScratchSpace>,
{
    match get_local_scratch() {
        Some(scratch) => Ok(scratch),
        None => {
            set_local_scratch(f());
            if let Some(scratch) = get_local_scratch() {
                Ok(scratch)
            } else {
                bail!("Not possible to set local scratch.")
            }
        }
    }
}
