#[cfg(feature = "stream")]
use super::bind_stream::*;
use super::binding::*;
use super::computed::*;
use super::traits::*;

use std::sync::*;

///
/// A `BindRef` references another binding without needing to know precisely
/// what kind of binding it is. It is read-only, so mostly useful for passing
/// a binding around, particularly for computed bindings. Create one with
/// `BindRef::from(binding)`.
///
/// Cloning a `BindRef` will create another reference to the same binding.
///
pub struct BindRef<Target> {
    reference: Arc<dyn Bound<Target>>,
}

impl<Value> Bound<Value> for BindRef<Value> {
    #[inline]
    fn get(&self) -> Value {
        self.reference.get()
    }
}

impl<Value> Changeable for BindRef<Value> {
    #[inline]
    fn when_changed(&self, what: Arc<dyn Notifiable>) -> Box<dyn Releasable> {
        self.reference.when_changed(what)
    }
}

impl<Value> Clone for BindRef<Value> {
    fn clone(&self) -> Self {
        BindRef {
            reference: Arc::clone(&self.reference),
        }
    }
}

impl<Value> BindRef<Value> {
    ///
    /// Creates a new BindRef from a reference to an existing binding
    ///
    #[inline]
    #[allow(dead_code)]
    pub fn new<Binding: 'static + Clone + Bound<Value>>(binding: &Binding) -> BindRef<Value> {
        BindRef {
            reference: Arc::new(binding.clone()),
        }
    }

    ///
    /// Creates a new BindRef from an existing binding
    ///
    #[inline]
    #[allow(dead_code)]
    pub fn from_arc<Binding: 'static + Bound<Value>>(binding_ref: Arc<Binding>) -> BindRef<Value> {
        BindRef {
            reference: binding_ref,
        }
    }
}

impl<'a, Value> From<&'a BindRef<Value>> for BindRef<Value> {
    #[inline]
    fn from(val: &'a BindRef<Value>) -> Self {
        BindRef::clone(val)
    }
}

impl<Value: 'static + Clone + Send + Sync + PartialEq> From<Binding<Value>> for BindRef<Value> {
    #[inline]
    fn from(val: Binding<Value>) -> Self {
        BindRef {
            reference: Arc::new(val),
        }
    }
}

impl<'a, Value: 'static + Clone + PartialEq + Sync + Send> From<&'a Binding<Value>>
    for BindRef<Value>
{
    #[inline]
    fn from(val: &'a Binding<Value>) -> Self {
        BindRef {
            reference: Arc::new(val.clone()),
        }
    }
}

impl<Value: 'static + Clone + PartialEq + Sync + Send, TFn> From<ComputedBinding<Value, TFn>>
    for BindRef<Value>
where
    TFn: 'static + Send + Sync + Fn() -> Value,
{
    #[inline]
    fn from(val: ComputedBinding<Value, TFn>) -> Self {
        BindRef {
            reference: Arc::new(val),
        }
    }
}

impl<'a, Value: 'static + Clone + PartialEq + Sync + Send, TFn>
    From<&'a ComputedBinding<Value, TFn>> for BindRef<Value>
where
    TFn: 'static + Send + Sync + Fn() -> Value,
{
    #[inline]
    fn from(val: &'a ComputedBinding<Value, TFn>) -> Self {
        BindRef {
            reference: Arc::new(val.clone()),
        }
    }
}

impl<'a, Value: 'static + Clone + PartialEq + Send + Sync + Into<Binding<Value>>> From<&'a Value>
    for BindRef<Value>
{
    #[inline]
    fn from(val: &'a Value) -> BindRef<Value> {
        let binding: Binding<Value> = val.into();
        BindRef::from(binding)
    }
}

#[cfg(feature = "stream")]
impl<Value: 'static + Clone + Send + Sync + PartialEq> From<StreamBinding<Value>>
    for BindRef<Value>
{
    #[inline]
    fn from(val: StreamBinding<Value>) -> Self {
        BindRef {
            reference: Arc::new(val),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn bindref_matches_core_value() {
        let bind = bind(1);
        let bind_ref = BindRef::from(bind.clone());

        assert!(bind_ref.get() == 1);

        bind.set(2);

        assert!(bind_ref.get() == 2);
    }

    #[test]
    fn bind_ref_from_value() {
        let bind_ref = BindRef::from(&1);

        assert!(bind_ref.get() == 1);
    }

    #[test]
    fn bind_ref_from_computed() {
        let bind_ref = BindRef::from(computed(|| 1));

        assert!(bind_ref.get() == 1);
    }

    #[test]
    fn bindref_matches_core_value_when_created_from_ref() {
        let bind = bind(1);
        let bind_ref = BindRef::new(&bind);

        assert!(bind_ref.get() == 1);

        bind.set(2);

        assert!(bind_ref.get() == 2);
    }
}
