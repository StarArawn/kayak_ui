use kayak_ui::core::derivative::*;
use kayak_ui::core::{rsx, widget, Children};

use crate::Clip;

#[widget]
pub fn App(children: Children) {
    rsx! {
        <Clip>
            {children}
        </Clip>
    }
}
