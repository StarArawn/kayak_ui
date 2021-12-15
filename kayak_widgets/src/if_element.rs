use kayak_ui::core::{rsx, widget, Children};

#[widget]
pub fn If(children: Children, condition: bool) {
    if *condition {
        rsx! {
            <>
                {children}
            </>
        }
    } else {
    }
}
