use kayak_core::{rsx, widget, Children};

use kayak_core::derivative::*;

#[widget]
pub fn App(children: Children) {
    rsx! {
        <>
            {children}
        </>
    }
}
