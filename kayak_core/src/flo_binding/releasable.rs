use super::traits::*;

use std::sync::*;

// TODO: issue with new 'drop' behaviour is what to do when we clone, as if keep_alive is
// false on the clone then dropping the clone will also drop this object. Sometimes we
// want this behaviour and sometimes we don't.

///
/// A notifiable that can be released (and then tidied up later)
///
pub struct ReleasableNotifiable {
    /// Set to true if this object should not release on drop. Note this is not shared,
    /// so the first ReleasableNotifiable in a group to be dropped where keep_alive
    /// is false will mark all the others as done.
    keep_alive: bool,

    /// The notifiable object that should be released when it's done
    target: Arc<Mutex<Option<Arc<dyn Notifiable>>>>
}

impl ReleasableNotifiable {
    ///
    /// Creates a new releasable notifiable object
    ///
    pub fn new(target: Arc<dyn Notifiable>) -> ReleasableNotifiable {
        ReleasableNotifiable {
            keep_alive: false,
            target:     Arc::new(Mutex::new(Some(target)))
        }
    }

    ///
    /// Marks this as changed and returns whether or not the notification was called
    ///
    pub fn mark_as_changed(&self) -> bool {
        // Get a reference to the target via the lock
        let target = {
            // Reset the optional item so that it's 'None'
            let target = self.target.lock().unwrap();

            // Send to the target
            target.clone()
        };

        // Send to the target
        if let Some(ref target) = target {
            target.mark_as_changed();
            true
        } else {
            false
        }
    }

    ///
    /// True if this item is still in use
    ///
    pub fn is_in_use(&self) -> bool {
        self.target.lock().unwrap().is_some()
    }

    ///
    /// Creates a new 'owned' clone (which will expire this notifiable when dropped)
    /// 
    pub fn clone_as_owned(&self) -> ReleasableNotifiable {
        ReleasableNotifiable {
            keep_alive: self.keep_alive,
            target:     Arc::clone(&self.target)
        }
    }

    ///
    /// Creates a new 'inspection' clone (which can be dropped without ending
    /// the lifetime of the releasable object)
    ///
    pub fn clone_for_inspection(&self) -> ReleasableNotifiable {
        ReleasableNotifiable {
            keep_alive: true,
            target:     Arc::clone(&self.target)
        }
    }
}

impl Releasable for ReleasableNotifiable {
    fn done(&mut self) {
        // Reset the optional item so that it's 'None'
        let mut target = self.target.lock().unwrap();

        *target = None;
    }

    fn keep_alive(&mut self) {
        self.keep_alive = true;
    }
}

impl Notifiable for ReleasableNotifiable {
    fn mark_as_changed(&self) {
        // Get a reference to the target via the lock
        let target = {
            // Reset the optional item so that it's 'None'
            let target = self.target.lock().unwrap();

            // Send to the target
            target.clone()
        };

        // Make sure we're calling out to mark_as_changed outside of the lock
        if let Some(target) = target {
            target.mark_as_changed();
        }
    }
}

impl Drop for ReleasableNotifiable {
    fn drop(&mut self) {
        if !self.keep_alive {
            self.done();
        }
    }
}

impl Releasable for Vec<Box<dyn Releasable>> {
    fn done(&mut self) {
        for item in self.iter_mut() {
            item.done();
        }
    }

    fn keep_alive(&mut self) {
        for item in self.iter_mut() {
            item.keep_alive();
        }
    }
}
