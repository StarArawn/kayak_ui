# Managing Input

The plugin already handles managing input for you. Input events are fed to the `KayakContext`, processed, and dispatched to their respective widgets.

What this *doesn't* do automatically is prevent other Bevy systems from using the input themselves. For example, pressing a button on your UI might also trigger your `move_player` system resulting in both the button being clicked and your player being moved.

Thankfully, Kayak has a built-in way of helping prevent this sort of thing. Using the `BevyContext` you can check whether the cursor is over the UI or not and respond accordingly.

```rust
use bevy::prelude::*;
use kayak_ui::bevy::BevyContext;
# 
# #[derive(Component)]
# struct Player;

fn move_player(
  player: Query<&mut Transform, With<Player>>,
  cursor: Res<Input<MouseButton>>,
  context: Res<BevyContext>
) {
  if(context.contains_cursor()) {
    // Cursor over UI -> skip this system
    return;
  }
}
```

> There's also the `wants_cursor` and `has_cursor` methods, which can be used to gain more information about how Kayak currently views the cursor
