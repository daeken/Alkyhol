//! Real asynchronous sample counters shared by overlapping D3D9 query scopes.
use crate::{Device, Result};
use std::sync::{Arc, Mutex, Weak, mpsc};
#[derive(Default)]
pub(crate) struct Tracker {
    active: Vec<Weak<Mutex<State>>>,
}
#[derive(Default)]
struct State {
    building: bool,
    issued: bool,
    generation: u64,
    tickets: Vec<Arc<Ticket>>,
    cached: Option<u32>,
}
/// Owned occlusion-query scope. Results are actual GPU samples, never estimates.
#[derive(Clone)]
pub struct OcclusionQuery {
    owner: Arc<Mutex<Tracker>>,
    state: Arc<Mutex<State>>,
}
struct Ticket {
    buffer: wgpu::Buffer,
    status: Mutex<TicketStatus>,
}
struct TicketStatus {
    receiver: mpsc::Receiver<std::result::Result<(), wgpu::BufferAsyncError>>,
    value: Option<u64>,
}
pub(crate) struct DrawQuery {
    pub(crate) set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    read: wgpu::Buffer,
    scopes: Vec<(Weak<Mutex<State>>, u64)>,
}
impl Ticket {
    fn value(&self) -> Result<Option<u64>> {
        let mut status = self
            .status
            .lock()
            .map_err(|_| "occlusion readback lock poisoned")?;
        if let Some(value) = status.value {
            return Ok(Some(value));
        }
        match status.receiver.try_recv() {
            Err(mpsc::TryRecvError::Empty) => return Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err("occlusion callback disconnected".into());
            }
            Ok(result) => result.map_err(|e| e.to_string())?,
        }
        let view = self.buffer.slice(..).get_mapped_range();
        let value = u64::from_le_bytes(view[..8].try_into().unwrap());
        drop(view);
        self.buffer.unmap();
        status.value = Some(value);
        Ok(Some(value))
    }
}
impl Device {
    pub fn create_occlusion_query(&self) -> OcclusionQuery {
        OcclusionQuery {
            owner: self.occlusion.clone(),
            state: Arc::new(Mutex::new(State::default())),
        }
    }
    fn query_owner(&self, query: &OcclusionQuery) -> Result<()> {
        if !Arc::ptr_eq(&self.occlusion, &query.owner) {
            return Err("occlusion query belongs to another device".into());
        }
        Ok(())
    }
    pub fn begin_occlusion_query(&self, query: &OcclusionQuery) -> Result<()> {
        self.query_owner(query)?;
        let mut tracker = self
            .occlusion
            .lock()
            .map_err(|_| "occlusion tracker lock poisoned")?;
        tracker.active.retain(|weak| weak.strong_count() != 0);
        if tracker.active.len() >= 128 {
            return Err("active occlusion query budget exhausted".into());
        }
        let mut state = query
            .state
            .lock()
            .map_err(|_| "occlusion scope lock poisoned")?;
        if state.building {
            return Err("occlusion query is already building".into());
        }
        state.generation = state
            .generation
            .checked_add(1)
            .ok_or("occlusion generation overflow")?;
        state.building = true;
        state.issued = false;
        state.cached = None;
        state.tickets.clear();
        tracker.active.push(Arc::downgrade(&query.state));
        Ok(())
    }
    pub fn end_occlusion_query(&self, query: &OcclusionQuery) -> Result<()> {
        self.query_owner(query)?;
        let mut tracker = self
            .occlusion
            .lock()
            .map_err(|_| "occlusion tracker lock poisoned")?;
        let mut state = query
            .state
            .lock()
            .map_err(|_| "occlusion scope lock poisoned")?;
        if !state.building {
            return Err("occlusion end requires a matching begin".into());
        }
        state.building = false;
        state.issued = true;
        let pointer = Arc::downgrade(&query.state);
        tracker
            .active
            .retain(|weak| !weak.ptr_eq(&pointer) && weak.strong_count() != 0);
        Ok(())
    }
    /// Nonblocking poll. None means actual GPU work is pending. Unissued data
    /// and count overflow fail explicitly rather than inventing a result.
    pub fn occlusion_data(&self, query: &OcclusionQuery, flush: bool) -> Result<Option<u32>> {
        self.query_owner(query)?;
        if flush {
            self.queue.submit([]);
        }
        self.check_status()?;
        let mut state = query
            .state
            .lock()
            .map_err(|_| "occlusion scope lock poisoned")?;
        if state.building {
            return Ok(None);
        }
        if !state.issued {
            return Err("unissued occlusion query data is not implemented".into());
        }
        if let Some(value) = state.cached {
            return Ok(Some(value));
        }
        let mut total = 0u64;
        for ticket in &state.tickets {
            let Some(value) = ticket.value()? else {
                return Ok(None);
            };
            total = total.checked_add(value).ok_or("occlusion count overflow")?;
        }
        let total = u32::try_from(total).map_err(|_| "occlusion DWORD count overflow")?;
        state.cached = Some(total);
        state.tickets.clear();
        Ok(Some(total))
    }
    pub(crate) fn draw_query(&self) -> Result<Option<DrawQuery>> {
        let mut tracker = self
            .occlusion
            .lock()
            .map_err(|_| "occlusion tracker lock poisoned")?;
        tracker.active.retain(|weak| weak.strong_count() != 0);
        if tracker.active.is_empty() {
            return Ok(None);
        }
        let mut scopes = Vec::new();
        for weak in &tracker.active {
            if let Some(scope) = weak.upgrade() {
                let state = scope.lock().map_err(|_| "occlusion scope lock poisoned")?;
                if state.tickets.len() >= 4096 {
                    return Err("occlusion draw budget exhausted".into());
                }
                scopes.push((weak.clone(), state.generation));
            }
        }
        let set = self.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("D3D9 actual occlusion"),
            ty: wgpu::QueryType::Occlusion,
            count: 1,
        });
        let resolve = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("D3D9 query resolve"),
            size: 256,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("D3D9 query readback"),
            size: 8,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Ok(Some(DrawQuery {
            set,
            resolve,
            read,
            scopes,
        }))
    }
}
impl DrawQuery {
    pub(crate) fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(&self.set, 0..1, &self.resolve, 0);
        encoder.copy_buffer_to_buffer(&self.resolve, 0, &self.read, 0, 8);
    }
    pub(crate) fn submitted(self) -> Result<()> {
        let (send, receive) = mpsc::channel();
        self.read
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = send.send(result);
            });
        let ticket = Arc::new(Ticket {
            buffer: self.read,
            status: Mutex::new(TicketStatus {
                receiver: receive,
                value: None,
            }),
        });
        for (weak, generation) in self.scopes {
            if let Some(scope) = weak.upgrade() {
                let mut state = scope.lock().map_err(|_| "occlusion scope lock poisoned")?;
                if state.generation == generation {
                    state.tickets.push(ticket.clone());
                }
            }
        }
        Ok(())
    }
}
