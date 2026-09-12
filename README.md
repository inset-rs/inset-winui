# inset-winui

WinUI 3's controls, Fluent 2 look and motion on the Inset framework, ported from Microsoft's open source (`../microsoft-ui-xaml`). Desktop first; WinUI itself adapts by window size rather than by device, so there is no separate mobile idiom.

Sibling checkouts this workspace expects: `../reveal-rs` (the framework, path dependencies), `../valo` (the renderer, on the branch reveal-rs is pinned to), `../microsoft-ui-xaml` (the spec).

```sh
cargo run -p winui_gallery
cargo test
WINUI_CAPTURE_DIR=$PWD/output cargo test -p winui_gallery   # export what the tests rendered
python3 tools/gen_resources.py ../microsoft-ui-xaml crates/inset-winui/src/theme/generated.rs Button ToggleSwitch CheckBox RadioButton ToggleButton HyperlinkButton RepeatButton Slider ToolTip TextBlock CornerRadius SplitView TabView ScrollBar NavigationView FlyoutPresenter NavigationBackButton TextBox PasswordBox CommandBarFlyout AppBarButton AcrylicBrush ProgressBar Expander InfoBar MenuFlyout DropDownButton SplitButton RadioMenuFlyoutItem ProgressRing InfoBadge MenuBar BreadcrumbBar
```

The gallery uses NavigationView with a destination and icon for each feature. Each page groups examples into cards with settings for exploring control behavior, including navigation modes, tab options, slider ranges, and selection states. Typography and Icons pages demonstrate the type ramp and bundled Fluent symbols; the Acrylic page compares the material with its opaque fallback. Visited examples retain their state, and a theme switch stays in the pane footer.

The gallery includes TextBox and PasswordBox (native editing, selection, clipboard commands, multiline input and password reveal), TabView (selection, closing, sizing, scrolling and in-window drag/reorder) and NavigationView (adaptive left panes, hierarchy and top overflow), alongside the earlier button, slider and SplitView ports. Native framework and host differences are recorded in the source folders’ `PORTING.md` files.

`AGENTS.md` has the porting rules; `crates/inset-winui/README.md` the shape of the code.

The gallery includes dedicated `SplitButton`, `ToggleSplitButton` and `MenuFlyout` pages with independent primary actions, checked choices and nested menus.

`MenuBar` groups application commands under headers whose ids remain stable across rebuilds, with hover switching, keyboard navigation and a dedicated gallery page.
