use kayak_core::{context::KayakContext, styles::Style, Children};
use kayak_core::{Fragment, KayakContextRef};
use kayak_render_macros::{rsx, use_state, widget, WidgetProps};

#[derive(WidgetProps, Clone, Default, Debug, PartialEq)]
#[allow(dead_code)]
struct TestProps {
    foo: u32,
    #[prop_field(Styles)]
    styles: Option<Style>,
    #[prop_field(Children)]
    children: Option<Children>,
    #[prop_field(OnEvent)]
    on_event: Option<kayak_core::OnEvent>,
}

#[widget]
fn Test(props: TestProps) {
    let _ = use_state!(props.foo);
    let children = props.get_children();
    rsx! {
        <>{children}</>
    };
}

fn main() {
    let mut context = KayakContext::new();
    {
        let mut context = KayakContextRef::new(&mut context, None);
        let foo = 10;
        let test_styles = Style::default();
        let children: Option<kayak_core::Children> = None;
        rsx! {
            <Fragment>
                <Test foo={10}>
                    <Test foo={1}>
                        <Test foo={5}>
                            <Test foo={foo} styles={Some(test_styles)}>
                                {}
                            </Test>
                        </Test>
                    </Test>
                </Test>
            </Fragment>
        };

        let foo = 10;
        let test_styles = Style::default();
        let children: Option<kayak_core::Children> = None;
        rsx! {
            <Fragment>
                <Test foo={foo} styles={Some(test_styles)}>
                    {}
                </Test>
                <Test foo={5} styles={Some(test_styles)}>
                    {}
                </Test>
            </Fragment>
        }
    }
}
