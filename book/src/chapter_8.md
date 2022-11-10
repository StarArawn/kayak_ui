# Chapter 8 - Prebuilt Widget
Kayak UI comes with a few pre-built widgets. Some of these widgets are highly customizable and others might be more rigidly defined. 

## 1. Core Visual Widgets
Core visual widgets are widgets that provide specific rendering commands to the rendering backend. These drive the look and feel of Kayak UI!

### Background
A background is a widget that renders a quad. It's style render command is hard coded to always be `RenderCommand::Quad`. In addition to rendering a quad the background widget can be customized to render a border, border radius, and other similar affects.

#### Props:
- KStyle
- KChildren 
- OnEvent

### Clip
Clips are special widgets that cause areas of the renderer to not draw pixels outside of their bounds. These can essentially be considered an inverse "mask" although they are always only rectangular in shape. A clip can be useful to keep content from spilling out of an existing area, in that regard they almost behave like the CSS overflow property.

#### Props:
- KStyle
- KChildren

### Image
Like the name implies this widget renders an image. The image that is rendered is an image loaded in by bevy. The size is controlled by styles. The widget also responds to border radius via an SDF mask.

#### Props:
- KImage(Handle<Image>) - This component accepts a bevy handle to an image asset.
- KStyle

### Nine Patch
The nine patch widget is a special widget designed to render a sliced UI image. Also know as 9-slicing. This 2D technique allows users to render UI images at multiple resolutions while maintaining a level of quality. The image in the middle is repeated.

#### Props:
- NinePatch
    - `handle`: A bevy handle to a nine patch image asset which.
    - `border`: This represents the area that is sliced into eight pieces along the edge of the image. The ninth piece is the middle which is repeated.
- KStyle
- OnEvent
- KChildren

### Text
This widget renders text directly to the screen.

#### Props:
- TextProps
    - `content`: The string to display
    - `font`: The name of the font to use 
    - `line_height`: The height of a line of text (currently in pixels). Defaults to font size * 1.2 which is the firefox default method of calculating line height.
    - `show_cursor`: If true, displays the default text cursor when hovered.
    - `size`: The font size (in pixels)
    - `alignement`: Text alignment.
    - `user_styles`: Specific styles applied directly to the text itself.
    - `word_wrap`: Wraps the words if said text would overflow it's parent.
- KStyle

### Texture Atlas
The texture atlas widget will render a bevy texture atlas inside of the UI. This can be useful for users who have UI that lives inside of an atlas texture. Although currently there are not any performance benefits of using this compared to just a regular single image.

#### Props
- TextureAtlasProps
    - `handle`: The handle to bevy texture atlas image
    - `position`: The position of the tile (in pixels)
    - `tile_size`: The size of the tile (in pixels)
- KStyle
