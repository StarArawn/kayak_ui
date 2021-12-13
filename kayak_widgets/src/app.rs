use kayak_ui::core::{rsx, widget, Children};
use kayak_ui::core::derivative::*;

#[widget]
pub fn App(children: Children) {
    rsx! {
        <>
            {children}
        </>
    }
}
