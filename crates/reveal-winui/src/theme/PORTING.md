# reveal-winui/src/theme
Audience: readers familiar with Flutter and Rust; the headings identify the corresponding WinUI code.

WinUI home: controls/dev/CommonStyles (`Common_themeresources_any.xaml`, `<Control>_themeresources.xaml`)
Ported against: aa3207e6

## mod.rs → accent palette and theme dictionaries

- Change: `AccentPalette` uses a supplied palette, defaulting to Windows 11 blue, instead of reading the user’s system accent color.
  Reason: os — the source obtains the accent shades from Windows, while this host supplies no system accent palette.
  Affect: `AccentPalette` does not follow system accent changes automatically; its default control colors are `dark1` (#0067C0) in light mode and `light2` (#4CC2FF) in dark mode, while `base`, `light1`, `light3`, `dark2` and `dark3` still need Windows verification.

- Change: `Theme` supports light and dark appearances but does not adapt to system high-contrast mode.
  Reason: os — WinUI obtains the user’s high-contrast colors from Windows, and this host does not expose that palette.
  Affect: `Theme` remains light or dark when the user enables system high contrast.

## typography.rs → `XamlAutoFontFamily`

- Change: `FONT_FAMILY` selects Selawik instead of Segoe UI Variable for control text.
  Reason: os — Segoe ships with Windows only; Selawik is Microsoft's metric-compatible open substitute.
  Affect: Control text has different letter shapes and does not automatically use the size-specific letter shapes supplied by Segoe UI Variable.

## Deferred

- Verification of all accent shades remains deferred. Trigger: A palette table is read from a Windows host.
- High-contrast theme support remains deferred. Trigger: A host reports the system palette.
