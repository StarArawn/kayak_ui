use kayak_core::{tree::Tree, widget, Arena, Index, Widget};

#[widget]
fn Widget1() {}

pub fn main() {
    let widget1 = Widget1 {
        id: Index::default(),
        styles: None,
        children: None,
        on_event: None,
    };

    let widget2 = Widget1 {
        id: Index::default(),
        styles: None,
        children: None,
        on_event: None,
    };

    let widget3 = Widget1 {
        id: Index::default(),
        styles: None,
        children: None,
        on_event: None,
    };

    let widget4 = Widget1 {
        id: Index::default(),
        styles: None,
        children: None,
        on_event: None,
    };

    // let widget5 = Widget1 {
    //     id: Index::default(),
    //     styles: None,
    //     children: None,
    //     on_event: None,
    // };

    let mut widgets = Arena::<Box<dyn Widget>>::default();
    let widget1_id = widgets.insert(Box::new(widget1));
    let widget2_id = widgets.insert(Box::new(widget2));
    let widget3_id = widgets.insert(Box::new(widget3));
    let widget4_id = widgets.insert(Box::new(widget4));
    // let widget5_id = widgets.insert(Box::new(widget5));

    let mut tree1 = Tree::default();
    tree1.add(widget1_id, None);
    tree1.add(widget2_id, Some(widget1_id));
    tree1.add(widget3_id, Some(widget2_id));
    // tree1.add(1, widget3_id, Some(widget1_id));
    // tree1.add(2, widget4_id, Some(widget1_id));

    let mut tree2 = Tree::default();
    tree2.add(widget1_id, None);
    tree2.add(widget2_id, Some(widget1_id));
    // tree2.add(0, widget3_id, Some(widget1_id));
    // tree2.add(1, widget4_id, Some(widget1_id));
    // tree2.add(2, widget5_id, Some(widget1_id));

    let changes = tree1.diff_children(&tree2, widget1_id);
    dbg!(&changes);
    tree1.merge(&tree2, widget1_id, changes);

    let changes = tree1.diff_children(&tree2, widget1_id);

    dbg!(&changes);

    assert!(tree1 == tree2);

    let mut tree1 = Tree::default();
    tree1.add(widget1_id, None);
    tree1.add(widget2_id, Some(widget1_id));
    tree1.add(widget3_id, Some(widget2_id));

    let mut tree2 = Tree::default();
    tree2.add(widget1_id, None);
    tree2.add(widget2_id, Some(widget1_id));
    tree2.add(widget4_id, Some(widget2_id));

    let changes = tree1.diff_children(&tree2, widget1_id);
    dbg!(&changes);

    tree1.merge(&tree2, widget1_id, changes);
    let differences = tree1.diff_children(&tree2, widget1_id);
    dbg!(differences);
}
