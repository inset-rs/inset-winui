# reveal-winui

Ported against: `microsoft/microsoft-ui-xaml` at `aa3207e6` (the Fluent styles in `controls/dev/CommonStyles`, the tokens in `Common_themeresources_any.xaml`, the framework in `dxaml/`). Entries are Change / Reason / Affect against that source.

Reasons are `os`, `framework` or `language`, as `AGENTS.md` defines them.

## theme/ → `ResourceDictionary.ThemeDictionaries`, `SystemAccentColor*`

- Change: the accent palette is a value (`AccentPalette`) with Windows 11's default blue built in; the OS supplies it to XAML apps.
  Reason: os — the shades are not in the repository, and there is no Windows to ask on this host.
  Affect: `dark1` (#0067C0) and `light2` (#4CC2FF), the shades light and dark controls actually use, are the published defaults; the other five shades are the ramp Windows shows for that blue and are unverified. Trigger: reading `UISettings` on a Windows host, or a verified table.

- Change: only the `Default` (dark) and `Light` dictionaries are generated; `HighContrast` is not.
  Reason: os — high contrast resolves to the OS's `SystemColor*` colours.
  Affect: no high-contrast theme. Trigger: a host that reports high contrast and its system colours.

- Change: text renders in Selawik, registered under `FONT_FAMILY`, instead of `XamlAutoFontFamily` (Segoe UI Variable).
  Reason: os — Segoe ships with Windows only; Selawik is Microsoft's metric-compatible open substitute.
  Affect: letterforms differ slightly; metrics match. No Segoe UI Variable optical sizes.

## primitives/ → `ContentPresenter`, `Border`, `BrushTransition`, `VisualStateManager`, system focus visuals

- Change: `Grid` layouts in templates are transcribed by hand into rows, columns and stacks.
  Reason: framework — reveal has no `Grid` with row and column definitions.
  Affect: none visible for the templates ported so far; a template with star-sized cells that a hand mapping cannot express would need a `Grid` widget.

- Change: `BrushTransition` interpolates linearly.
  Reason: os — the transition names a duration and no easing; the easing is the composition layer's.
  Affect: the 83 ms background fade is linear.

- Change: the system focus visual's colours are fixed to black / white (primary) and white / black at 60 % (secondary).
  Reason: os — XAML takes them from the OS's `SystemBaseHighColor` and `SystemAltMediumColor`.
  Affect: matches Windows defaults; ignores a user's high-contrast palette.

## controls/ → `Button`, `ToggleSwitch`

- Change: event handlers are closures (`Button::new(content, click)`, `ToggleSwitch::new(is_on, toggled)`) and state is the caller's; there are no dependency properties or bindings.
  Reason: language — XAML's `IsOn` is a dependency property the control writes back to; here the `toggled` handler receives the value the switch wants and the caller decides.
  Affect: a `ToggleSwitch` whose handler ignores the value stays where it is, as a two-way binding to a rejecting setter would.


- Change: the knob travels over `ControlFastAnimationDuration` (167 ms) with `ControlFastOutSlowInKeySpline`.
  Reason: os — the template uses `RepositionThemeAnimation`, whose timing the OS animation library (`uxtheme`) provides at run time.
  Affect: the slide may differ from Windows by tens of milliseconds and in curve. Trigger: a recording from a Windows machine.

## Deferred

- `ToggleSwitch` dragging (`SwitchThumb`, `MoveDelta`, the half-way toggle rule in `ToggleSwitch_Partial.cpp`). Trigger: a horizontal drag recogniser wired into `CommonStates`.
- The `HeaderContentPresenter` deferred-load semantics; `OnContent` / `OffContent` crossfade (`ContentStates`, discrete in the source, so nothing is lost yet).
- `RepeatButton`, `HyperlinkButton`, `ToggleButton`, `CheckBox`, `RadioButton`, `Slider`, `ProgressBar`, `ProgressRing`, `TextBlock` styles beyond the ramp, `Expander`, `InfoBar`, `NavigationView`, `TabView`, flyouts and dialogs: not started.
- Mica and Acrylic backdrops (`controls/dev/Materials`): the Acrylic effect graph is in the source; Mica's wallpaper processing is not.
- Text input controls: the editing engine is RichEdit, outside the source.
