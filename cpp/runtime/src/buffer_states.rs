//! Transactional DEFAULT-buffer state validation for serial Metal blit workloads.
//! Resource locks cover the whole ExecuteCommandLists batch, never partial lists.
use super::*;
use std::sync::atomic::AtomicUsize;
pub(super) struct Access {
    resources: Vec<Arc<Mutex<BufferUsage>>>,
    previous: Vec<u32>,
    remaining: AtomicUsize,
    committed: AtomicBool,
    failed: AtomicBool,
}
impl Access {
    pub(super) fn commit(&self) {
        self.committed.store(true, Ordering::Release);
    }
    pub(super) fn complete_member(&self, failed: bool) {
        if failed {
            self.failed.store(true, Ordering::Release);
        }
        if self.remaining.fetch_sub(1, Ordering::AcqRel) != 1 {
            return;
        }
        for (resource, &previous) in self.resources.iter().zip(&self.previous) {
            let mut r = resource.lock().unwrap();
            if !self.committed.load(Ordering::Acquire) {
                r.current = previous;
            }
            r.failed |= self.failed.load(Ordering::Acquire);
            r.pending -= 1;
            if r.pending == 0 {
                r.owner = None;
            }
        }
    }
}
#[derive(Clone, Copy)]
struct Simulated {
    bits: u32,
    promoted: bool,
}
impl Simulated {
    fn access(&mut self, required: u32) -> Result<(), i32> {
        if self.bits == 0 {
            self.bits = required;
            self.promoted = true;
            return Ok(());
        }
        if self.bits & required == required {
            return Ok(());
        }
        if self.promoted && self.bits & !0xac3 == 0 && required & !0xac3 == 0 {
            self.bits |= required;
            return Ok(());
        }
        Err(E_INVALIDARG)
    }
    fn transition(&mut self, before: u32, after: u32) -> Result<(), i32> {
        // COMMON also accepts a promotable BeforeState, per the documented
        // buffer-decay contract. An explicit state otherwise must agree.
        if self.bits != 0 && self.bits != before {
            return Err(E_INVALIDARG);
        }
        self.bits = after;
        self.promoted = false;
        Ok(())
    }
}
fn key(resource: &Arc<Mutex<BufferUsage>>) -> usize {
    Arc::as_ptr(resource) as usize
}
pub(super) fn reserve(
    batches: &[Arc<Submission>],
    owner: &Arc<()>,
) -> Result<Option<Arc<Access>>, i32> {
    let mut resources = Vec::new();
    for batch in batches {
        for command in &batch.commands {
            resources.try_reserve(2).map_err(|_| E_OUTOFMEMORY)?;
            match command {
                Command::Copy(c) => {
                    resources.extend(c.source_usage.iter().cloned());
                    resources.extend(c.destination_usage.iter().cloned());
                }
                Command::Transition(t) => resources.push(Arc::clone(&t.usage)),
            }
        }
    }
    resources.sort_unstable_by_key(key);
    resources.dedup_by(|a, b| Arc::ptr_eq(a, b));
    if resources.is_empty() {
        return Ok(None);
    }
    let mut locks = Vec::new();
    locks
        .try_reserve(resources.len())
        .map_err(|_| E_OUTOFMEMORY)?;
    for r in &resources {
        locks.push(r.lock().unwrap());
    }
    for r in &locks {
        if r.failed {
            return Err(0x887a0005u32 as i32);
        }
        if r.pending != 0
            && r.owner
                .as_ref()
                .is_none_or(|other| !Arc::ptr_eq(other, owner))
        {
            return Err(E_NOTIMPL);
        }
        if r.pending == usize::MAX {
            return Err(E_OUTOFMEMORY);
        }
    }
    let previous: Vec<u32> = locks.iter().map(|r| r.current).collect();
    let mut simulated: Vec<Simulated> = previous
        .iter()
        .map(|&bits| Simulated {
            bits,
            promoted: false,
        })
        .collect();
    let index =
        |usage: &Arc<Mutex<BufferUsage>>| resources.binary_search_by_key(&key(usage), key).unwrap();
    for batch in batches {
        for command in &batch.commands {
            match command {
                Command::Copy(c) => {
                    if let Some(usage) = &c.source_usage {
                        simulated[index(usage)].access(0x800)?;
                    }
                    if let Some(usage) = &c.destination_usage {
                        simulated[index(usage)].access(0x400)?;
                    }
                }
                Command::Transition(t) => {
                    simulated[index(&t.usage)].transition(t.before, t.after)?
                }
            }
        }
    }
    // Decay occurs at the ExecuteCommandLists boundary, not between lists in
    // this batch. This is the scheduled state for later same-queue workloads;
    // pending ownership rejects cross-queue use before completion notification.
    for r in &mut locks {
        r.current = 0;
        r.pending += 1;
        r.owner = Some(Arc::clone(owner));
    }
    drop(locks);
    Ok(Some(Arc::new(Access {
        resources,
        previous,
        remaining: AtomicUsize::new(batches.len()),
        committed: AtomicBool::new(false),
        failed: AtomicBool::new(false),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_promotion_combines_reads_but_never_promotes_a_write_into_a_read() {
        let mut state = Simulated {
            bits: 0,
            promoted: false,
        };
        assert_eq!(state.access(1), Ok(()));
        assert_eq!(state.access(0x800), Ok(()));
        assert_eq!(state.bits, 0x801);
        assert_eq!(state.access(0x400), Err(E_INVALIDARG));
        let mut write = Simulated {
            bits: 0,
            promoted: false,
        };
        assert_eq!(write.access(0x400), Ok(()));
        assert_eq!(write.access(0x800), Err(E_INVALIDARG));
        let mut explicit = Simulated {
            bits: 1,
            promoted: false,
        };
        assert_eq!(explicit.access(0x800), Err(E_INVALIDARG));
    }
    #[test]
    fn explicit_transitions_validate_before_state_and_accept_promotable_before_after_decay() {
        let mut state = Simulated {
            bits: 0x400,
            promoted: false,
        };
        assert_eq!(state.transition(0x800, 1), Err(E_INVALIDARG));
        assert_eq!(state.bits, 0x400);
        assert_eq!(state.transition(0x400, 0x800), Ok(()));
        assert_eq!(state.access(0x800), Ok(()));
        state.bits = 0;
        assert_eq!(state.transition(0x800, 0x400), Ok(()));
        assert!(!state.promoted);
    }
}
