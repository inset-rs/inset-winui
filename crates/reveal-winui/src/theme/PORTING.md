# reveal-winui/src/theme
WinUI home: controls/dev/CommonStyles (`Common_themeresources_any.xaml`, `<Control>_themeresources.xaml`)
Ported against: aa3207e6

## mod.rs → the accent palette, the theme dictionaries

- Change: `AccentPalette` is a value with Windows 11's default blue built in.
  Reason: os — the shades are not in the repository, and there is no Windows to ask on this host.
  Affect: the two shades controls use are the published defaults; the other five are unverified. Trigger: a table read from a Windows host.

- Change: `Theme` has no high-contrast member.
  Reason: os — high contrast resolves to the OS's system colours.
  Affect: high contrast is ignored. Trigger: a host that reports it.

## typography.rs → `XamlAutoFontFamily`

- Change: `FONT_FAMILY` is Selawik instead of Segoe UI Variable.
  Reason: os — Segoe ships with Windows only; Selawik is Microsoft's metric-compatible open substitute.
  Affect: letterforms differ slightly; metrics match; no optical sizes.
