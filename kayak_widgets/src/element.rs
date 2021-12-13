use kayak_core::{component, rsx, Render, Update};

#[component]
pub fn Element<Children: Render + Update + Clone>(children: Children) {
    rsx! {
        <>
            {children}
        </>
    }
}
