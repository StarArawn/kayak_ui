use kayak_ui::core::{constructor, rsx, widget, Handler, VecTracker};
use kayak_ui::widgets::Element;

use super::{card::Card, Todo};

#[widget]
pub fn Cards(cards: Vec<Todo>, on_delete: Handler<usize>) {
    rsx! {
        <Element>
            {VecTracker::from(
                cards
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, todo)| constructor! { <Card card_id={index} name={todo.name.clone()} on_delete={on_delete.clone()} /> }),
            )}
        </Element>
    }
}
