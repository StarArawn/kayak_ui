# Keyboard Events

This section pertains to events related to the keyboard.

## Event Types

### Key Down

---

**Propagates** - ✅

**Inner Type** - `KeyboardEvent`

This event is triggered when a key is pressed down. It targets the currently focused widget.

This may also be repeated if the key is held down, depending on the operating system and its configuration.

#### Default Actions

| Key Pressed                     | Action                                     |
| ------------------------------- | ------------------------------------------ |
| <kbd>Tab</kbd>                  | Navigates to the next focusable widget     |
| <kbd>Shift</kbd>+<kbd>Tab</kbd> | Navigates to the previous focusable widget |

### Key Up

---

**Propagates** - ✅

**Inner Type** - `KeyboardEvent`

This event is triggered when a key is released. It targets the currently focused widget.

### Char Input

---

**Propagates** - ✅

**Inner Type** - `{ c: char }`

This event is triggered when a character is received. It targets the currently focused widget. 

This may also be repeated if the character is held down, depending on the operating system and its configuration.
