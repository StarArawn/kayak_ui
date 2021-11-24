use kayak_core::{context::KayakContext, styles::Style, Children, Index};
use kayak_core::{derivative::*, Fragment, Widget};
use kayak_render_macros::rsx;

#[derive(Derivative)]
#[derivative(Debug, PartialEq)]
#[allow(dead_code)]
struct Test {
    id: Index,
    styles: Option<Style>,
    foo: u32,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    children: Children,
}

impl Widget for Test {
    fn get_id(&self) -> Index {
        todo!()
    }

    fn set_id(&mut self, _id: Index) {
        todo!()
    }

    fn get_styles(&self) -> Option<Style> {
        todo!()
    }

    fn render(&mut self, _context: &mut KayakContext) {
        todo!()
    }
}

fn main() {
    let mut context = KayakContext::new();
    {
        let context = &mut context;
        let foo = 10;
        let test_styles = Style::default();
        let parent_id: Option<Index> = None;
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

        let parent_id: Option<Index> = None;
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
