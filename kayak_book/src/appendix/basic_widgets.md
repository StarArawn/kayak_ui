# Built-in Widgets

TODO: Maybe flesh this out with individual sections where an example and (optional) screenshot can be added

In Kayak UI we have a set of default widgets you can use that are completely optional. A full list of widgets:

- [App](../../src/widgets/app.rs) - A top level widget that sizes your UI to the bevy screen width/height.
- [Background](../../src/widgets/background.rs) - A widget which renders a colored quad. 
- [Button](../../src/widgets/button.rs) - A simple button widget
- [Clip](../../src/widgets/clip.rs) - This widget provides a rectangular clip area. Widgets rendered as children of the Clip will get chopped off if they exceed the bounds of the clip.
- [Element](../../src/widgets/element.rs) - A widget who provides additional layout but renders nothing.
- [Fold](../../src/widgets/fold.rs) - A collapsing widget
- [If](../../src/widgets/if_element.rs) - A conditional widget that will render it's children depending on X condition.
- [Image](../../src/widgets/image.rs) - Renders the provided image.
- [Inspector](../../src/widgets/inspector.rs) - A special widget which allows you to "inspect" the tree by clicking on widgets.
- [NinePatch](../../src/widgets/nine_patch.rs) - A nine patch renderer widget. Nine patches are nine textures(atlas single image) that represent 9 different sections of a rectangle: corners, in-between corners, and the middle.
- [TextBox](../../src/widgets/text_box.rs) - A widget which allows typing.
- [Text](../../src/widgets/text.rs) - A widget that displays text.
- [Tooltip](../../src/widgets/tooltip.rs) - A widget that renders a popup that displays its children.
- [Window](../../src/widgets/window.rs) - A draggable box with a title and children.
