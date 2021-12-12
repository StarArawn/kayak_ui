#[test]
fn flo_binding_test() {
    use flo_binding::{Binding, Bound, Changeable, MutableBound};

    #[derive(Clone, PartialEq)]
    struct TestState {
        pub value: u32,
    }

    let mut some_state = resources::Resources::default();

    let test_state = flo_binding::bind(TestState { value: 0 });

    let mut lifetime = test_state.when_changed(flo_binding::notify(|| {
        dbg!("Changed!");
    }));

    some_state.insert(test_state);

    if let Ok(test_state) = some_state.get::<Binding<TestState>>() {
        assert!(test_state.get().value == 0);

        test_state.set(TestState { value: 2 });

        assert!(test_state.get().value == 2);
 
        lifetime.done();
    };

    panic!();
}
