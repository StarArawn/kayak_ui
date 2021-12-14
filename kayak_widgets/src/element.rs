use kayak_ui::core::{rsx, widget, Children};

#[widget]
pub fn Element(children: Children) {
    rsx! {
        <>
            {children}
        </>
    }
}
