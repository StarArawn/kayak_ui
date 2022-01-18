use super::traits::*;

use std::sync::*;

struct NotifyFn<TFn> {
    when_changed: Mutex<TFn>,
}

impl<TFn> Notifiable for NotifyFn<TFn>
where
    TFn: Send + FnMut() -> (),
{
    fn mark_as_changed(&self) {
        let on_changed = &mut *self.when_changed.lock().unwrap();

        on_changed()
    }
}

///
/// Creates a notifiable reference from a function
///
pub fn notify<TFn>(when_changed: TFn) -> Arc<dyn Notifiable>
where
    TFn: 'static + Send + FnMut() -> (),
{
    Arc::new(NotifyFn {
        when_changed: Mutex::new(when_changed),
    })
}
