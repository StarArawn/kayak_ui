# Chapter 5 - Fonts
Kayak UI uses SDF(signed distance fields) for rendering fonts. More specifically it uses multi-channel signed distance fields. Reasons for why we use MSDF:
- High Quality font rendering.
- Fast rendering!
- No need for a new asset for each font size. MSDF's can size to any font size!

Font's are stored as an atlased image and a json file which tells Kayak about the font glyphs. Check out `roboto.kayak_font` and `roboto.png` in the `assets` folder.

## Generating new fonts.
In order to create a new font you need to use the `msdf-atlas-gen` tool. This can be found at:
[https://github.com/Chlumsky/msdf-atlas-gen](https://github.com/Chlumsky/msdf-atlas-gen)

Please refer to the documentation found in the link about for generating fonts. However a simple way is to use the following command line:

```
.\msdf-atlas-gen.exe -font .\my_font.ttf -type msdf -minsize 32 -format png -imageout my_font.png -json my_font.kayak_font
```