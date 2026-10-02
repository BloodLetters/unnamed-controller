use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

/// Discrete simulation timestamp.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct SimTime(pub u64);

impl SimTime {
    /// Returns the zero simulation time.
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Adds discrete ticks to the simulation time.
    pub fn add_ticks(self, ticks: u64) -> Self {
        Self(self.0.saturating_add(ticks))
    }
}

/// Unique identifier for a scheduled event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EventId(pub u64);

/// Internal wrapper for heap ordering.
#[derive(Debug, Clone)]
struct HeapEntry<T> {
    id: EventId,
    time: SimTime,
    order: u64,
    payload: T,
}

impl<T> PartialEq for HeapEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.order == other.order
    }
}

impl<T> Eq for HeapEntry<T> {}

impl<T> PartialOrd for HeapEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for HeapEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .time
            .cmp(&self.time)
            .then_with(|| other.order.cmp(&self.order))
    }
}

/// Priority-queue based event scheduler providing cycle-accurate discrete event simulation.
pub struct EventScheduler<T> {
    heap: BinaryHeap<HeapEntry<T>>,
    cancelled: HashSet<EventId>,
    next_id: u64,
    order_counter: u64,
    current_time: SimTime,
}

impl<T> Default for EventScheduler<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EventScheduler<T> {
    /// Creates a new, empty event scheduler starting at time zero.
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            cancelled: HashSet::new(),
            next_id: 1,
            order_counter: 0,
            current_time: SimTime::zero(),
        }
    }

    /// Returns the current simulation time.
    pub fn current_time(&self) -> SimTime {
        self.current_time
    }

    /// Sets the current simulation time directly.
    pub fn set_current_time(&mut self, time: SimTime) {
        self.current_time = time;
    }

    /// Advances the current simulation time by the specified number of ticks.
    pub fn advance_time(&mut self, ticks: u64) {
        self.current_time = self.current_time.add_ticks(ticks);
    }

    /// Schedules a future event at the specified timestamp.
    pub fn schedule_at(&mut self, time: SimTime, payload: T) -> EventId {
        let id = EventId(self.next_id);
        self.next_id += 1;
        self.order_counter += 1;

        self.heap.push(HeapEntry {
            id,
            time,
            order: self.order_counter,
            payload,
        });

        id
    }

    /// Schedules an event relative to the current simulation time.
    pub fn schedule_after(&mut self, delta_ticks: u64, payload: T) -> EventId {
        let target = self.current_time.add_ticks(delta_ticks);
        self.schedule_at(target, payload)
    }

    /// Cancels a previously scheduled event by its ID.
    pub fn cancel(&mut self, id: EventId) -> bool {
        self.cancelled.insert(id)
    }

    /// Returns the timestamp of the next pending event without popping it.
    pub fn peek_next_time(&mut self) -> Option<SimTime> {
        self.prune_cancelled();
        self.heap.peek().map(|e| e.time)
    }

    /// Pops the next non-cancelled event from the priority queue.
    pub fn pop_next(&mut self) -> Option<(EventId, SimTime, T)> {
        while let Some(entry) = self.heap.pop() {
            if self.cancelled.remove(&entry.id) {
                continue;
            }
            if entry.time > self.current_time {
                self.current_time = entry.time;
            }
            return Some((entry.id, entry.time, entry.payload));
        }
        None
    }

    /// Collects and returns all pending events scheduled at or before the specified timestamp.
    pub fn pop_due(&mut self, target_time: SimTime) -> Vec<(EventId, T)> {
        let mut due = Vec::new();
        while let Some(entry) = self.heap.peek() {
            if entry.time > target_time {
                break;
            }
            let item = self.heap.pop().unwrap();
            if !self.cancelled.remove(&item.id) {
                due.push((item.id, item.payload));
            }
        }
        if target_time > self.current_time {
            self.current_time = target_time;
        }
        due
    }

    /// Returns the number of events currently queued.
    pub fn len(&self) -> usize {
        self.heap.len().saturating_sub(self.cancelled.len())
    }

    /// Queries whether the scheduler has no pending events.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Cleans up cancelled entries at the top of the heap.
    fn prune_cancelled(&mut self) {
        while let Some(entry) = self.heap.peek() {
            if self.cancelled.contains(&entry.id) {
                let removed = self.heap.pop().unwrap();
                self.cancelled.remove(&removed.id);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_ordering() {
        let mut scheduler = EventScheduler::new();
        scheduler.schedule_at(SimTime(100), "event_100");
        scheduler.schedule_at(SimTime(20), "event_20");
        scheduler.schedule_at(SimTime(50), "event_50");

        assert_eq!(scheduler.peek_next_time(), Some(SimTime(20)));
        let (_, time, payload) = scheduler.pop_next().unwrap();
        assert_eq!(time, SimTime(20));
        assert_eq!(payload, "event_20");

        let (_, time, payload) = scheduler.pop_next().unwrap();
        assert_eq!(time, SimTime(50));
        assert_eq!(payload, "event_50");

        let (_, time, payload) = scheduler.pop_next().unwrap();
        assert_eq!(time, SimTime(100));
        assert_eq!(payload, "event_100");

        assert!(scheduler.pop_next().is_none());
    }

    #[test]
    fn test_event_cancellation() {
        let mut scheduler = EventScheduler::new();
        let id1 = scheduler.schedule_at(SimTime(10), "first");
        let _id2 = scheduler.schedule_at(SimTime(20), "second");

        scheduler.cancel(id1);

        let (_, time, payload) = scheduler.pop_next().unwrap();
        assert_eq!(time, SimTime(20));
        assert_eq!(payload, "second");
        assert!(scheduler.pop_next().is_none());
    }

    #[test]
    fn test_pop_due() {
        let mut scheduler = EventScheduler::new();
        scheduler.schedule_at(SimTime(10), "e10");
        scheduler.schedule_at(SimTime(20), "e20");
        scheduler.schedule_at(SimTime(30), "e30");

        let due = scheduler.pop_due(SimTime(20));
        assert_eq!(due.len(), 2);
        assert_eq!(due[0].1, "e10");
        assert_eq!(due[1].1, "e20");
        assert_eq!(scheduler.current_time(), SimTime(20));

        let remaining = scheduler.pop_due(SimTime(50));
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].1, "e30");
        assert_eq!(scheduler.current_time(), SimTime(50));
    }
}
