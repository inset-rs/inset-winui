# reveal-winui/src/theme
WinUI home: controls/dev/CommonStyles (`Common_themeresources_any.xaml`, `<Control>_themeresources.xaml`)
Ported against: aa3207e6

## mod.rs → accent palette and theme dictionaries

- Change: `AccentPalette` is a value with Windows 11's default blue built in.
  Reason: os — the shades are not in the repository, and there is no Windows to ask on this host.
  Affect: `AccentPalette`’s two shades used by controls match the published defaults, while the other five remain unverified.

- Change: `Theme` has no high-contrast member.
  Reason: os — high contrast resolves to the OS's system colours.
  Affect: `Theme` ignores high contrast.

## typography.rs → `XamlAutoFontFamily`

- Change: `FONT_FAMILY` is Selawik instead of Segoe UI Variable.
  Reason: os — Segoe ships with Windows only; Selawik is Microsoft's metric-compatible open substitute.
  Affect: `FONT_FAMILY` letterforms differ slightly, metrics match, and optical sizes are unavailable.

## Deferred

- Verification of all accent shades remains deferred. Trigger: A palette table is read from a Windows host.
- High-contrast theme support remains deferred. Trigger: A host reports the system palette.
