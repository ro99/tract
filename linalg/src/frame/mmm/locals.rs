use tract_data::anyhow::{self, bail, Error};

use super::ScratchSpace;
use std::{cell::UnsafeCell, thread::LocalKey};

type ScratchLts = UnsafeCell<Option<Box<dyn ScratchSpace>>>;

thread_local!(
    static LOCAL_SCRATCH: ScratchLts = UnsafeCell::new(Option::None);
);

lazy_static::lazy_static!(
    pub static ref POOL: rayon::ThreadPool = rayon::ThreadPoolBuilder::new().build().unwrap();
);

pub struct LocalScratch {
    thread_local: &'static LocalKey<ScratchLts>,
}

impl LocalScratch {


    pub fn new() -> Self {
        Self { thread_local: &LOCAL_SCRATCH }
    }

    fn unset_local_scratch() {
        LOCAL_SCRATCH.with(move |e| unsafe {
            *e.get() = None;
        });
    }

    pub fn finish() {
        Self::unset_local_scratch();
    }

    fn get_local_scratch(&self) -> &mut Option<Box<dyn ScratchSpace>> {
        self.thread_local.with(move |e| unsafe { &mut *e.get() })
    }

    fn set_local_scratch(&self, scratch: Box<dyn ScratchSpace>) {
        self.thread_local.with(move |e| unsafe {
            *e.get() = Some(scratch);
        });
    }

    pub fn init<F>(&self, f: F) -> anyhow::Result<&mut Box<dyn ScratchSpace>, Error>
    where
        F: FnOnce() -> Box<dyn ScratchSpace>,
    {
        self.set_local_scratch(f());
        if let Some(scratch) = self.get_local_scratch() {
            Ok(scratch)
        } else {
            bail!("unreachable")
        }
    }

    /// Only call this if we are sure that init was called for this thread!
    pub fn get(&self) -> anyhow::Result<&mut Box<dyn ScratchSpace>, Error> {
        if let Some(scratch) = self.get_local_scratch() {
            Ok(scratch)
        } else {
            bail!("panic!")
        }
    }

    pub fn get_or_init<F>(&self, f: F) -> anyhow::Result<&mut Box<dyn ScratchSpace>, Error>
    where
        F: FnOnce() -> Box<dyn ScratchSpace>,
    {
        match self.get_local_scratch() {
            Some(scratch) => Ok(scratch),
            None => self.init(|| f()),
        }
    }
}
