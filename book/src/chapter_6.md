# Chapter 6 - Fonts
Kayak UI uses SDF(signed distance fields) for rendering fonts. More specifically it uses multi-channel signed distance fields. Reasons for why we use MSDF:
- High Quality font rendering.
- Fast rendering!
- No need for a new asset for each font size. MSDF's can size to any font size!

Fonts are stored in two different ways. First a font can be defined as a Kayak TTF(kttf) file. 
These font files are relatively simple and simply link to a ttf font:
```json
{
    "file": "roboto.ttf",
    "char_range_start": "0x20",
    "char_range_end": "0x7f"
}
``` 
The char range is a defined as u32 char values. 0x20 through 0x7f represents most of the standard English language characters. Font's using this method are processed in native rust into MSDF's. The output is cached as the generation can take a while. 

Fonts are also stored as an atlased image and a json file which tells Kayak about the font glyphs. These fonts are generated using `msdf-atlas-gen`. Check out `roboto.kayak_font` and `roboto.png` in the `assets` folder. The cached file name will be located next to the kttf file and have the file format of: `{font_name}.kttf-cached.png`.

### Generating Legacy `*.kayak_font`. WARNING! Does not work in wasm.
In order to create a new font you need to use the `msdf-atlas-gen` tool. This can be found at:
[https://github.com/Chlumsky/msdf-atlas-gen](https://github.com/Chlumsky/msdf-atlas-gen)

Please refer to the documentation found in the link about for generating fonts. However a simple way is to use the following command line:

```
.\msdf-atlas-gen.exe -font .\my_font.ttf -type msdf -minsize 32 -format png -imageout my_font.png -json my_font.kayak_font
```
