use std::time::Instant;

pub use flo_binding::{bind, notify, Binding, Bound, Changeable, MutableBound, Releasable};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Debouncer {
    last_updated: Instant,
    time: f32,
}

impl Debouncer {
    pub fn new(time: f32) -> Self {
        Self {
            time,
            last_updated: Instant::now(),
        }
    }

    pub fn should_update(&mut self) -> bool {
        let elapsed_time = self.last_updated.elapsed().as_secs_f32();
        if elapsed_time > self.time {
            self.last_updated = Instant::now();

            return true;
        }

        return false;
    }
}
