use kayak_ui::core::{constructor, rsx, widget, Handler, VecTracker, WidgetProps};
use kayak_ui::widgets::Element;

use super::{card::Card, Todo};

#[derive(WidgetProps, Clone, Debug, Default, PartialEq)]
pub struct CardsProps {
    pub cards: Vec<Todo>,
    pub on_delete: Handler<usize>,
}

#[widget]
pub fn Cards(props: CardsProps) {
    let CardsProps { cards, on_delete } = props.clone();
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
