//! WinUI's theme resources: the XAML `ResourceDictionary.ThemeDictionaries` (light, dark) and the OS accent palette, reachable from any widget through [`ThemeResources::of`] the way a template reaches `{ThemeResource X}`.

mod generated;
mod typography;
pub use generated::*;
pub use typography::*;

use inset_embedder::Color;
use inset_foundation::App;
use inset_widgets::{BuildContext, InheritedWidget, IntoWidget, MediaQuery, WidgetRef};

/// The source AcrylicBrush tint, luminosity, and fallback recipe.
/// `Brush::Acrylic` paints the in-window material using these values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AcrylicBrushResources {
    /// The colour blended over the blurred backdrop.
    pub tint_color: Color,
    /// The opacity applied to the tint colour.
    pub tint_opacity: f64,
    /// An explicit luminosity-layer opacity, or the source's automatic calculation.
    pub tint_luminosity_opacity: Option<f64>,
    /// The source colour used when acrylic composition is unavailable.
    pub fallback_color: Color,
}

/// XAML `ElementTheme` / the theme dictionary a lookup resolves in. High contrast is deferred: its values are the OS's system colours.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

/// The `SystemAccentColor*` resources the OS supplies to every XAML app: the accent and its three lighter and three darker shades. Light theme controls use `dark1`, dark theme controls `light2`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccentPalette {
    pub base: Color,
    pub light1: Color,
    pub light2: Color,
    pub light3: Color,
    pub dark1: Color,
    pub dark2: Color,
    pub dark3: Color,
}

impl AccentPalette {
    /// Windows 11's default blue. `dark1` (light-theme controls, #0067C0) and `light2` (dark-theme controls, #4CC2FF) are the published defaults; the other shades are the ramp Windows shows for that blue and should be verified against a Windows machine.
    pub const WINDOWS_BLUE: AccentPalette = AccentPalette {
        base: Color::from_argb(255, 0, 120, 212),
        light1: Color::from_argb(255, 0, 145, 248),
        light2: Color::from_argb(255, 76, 194, 255),
        light3: Color::from_argb(255, 153, 235, 255),
        dark1: Color::from_argb(255, 0, 103, 192),
        dark2: Color::from_argb(255, 0, 62, 146),
        dark3: Color::from_argb(255, 0, 26, 104),
    };
}

impl Default for AccentPalette {
    fn default() -> Self {
        Self::WINDOWS_BLUE
    }
}

/// The resolved theme a subtree renders with: the theme, the accent palette, and the shared Fluent tokens for that theme. Controls fetch their own `<Control>Resources` from it.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeResources {
    pub theme: Theme,
    pub accent: AccentPalette,
    pub common: CommonResources,
}

impl ThemeResources {
    pub fn new(theme: Theme, accent: AccentPalette) -> ThemeResources {
        ThemeResources {
            theme,
            accent,
            common: CommonResources::for_theme(theme, &accent),
        }
    }

    /// The nearest [`ThemeScope`]'s resources, or the platform's (light or dark from the media query, the default accent) when none encloses `context`.
    pub fn of(app: &mut App, context: BuildContext) -> ThemeResources {
        if let Some(scope) = context.depend_on_inherited_widget_of_exact_type::<ThemeScope>(app) {
            return scope.resources.clone();
        }
        let theme = match MediaQuery::maybe_platform_brightness_of(app, context) {
            Some(inset_embedder::Brightness::Dark) => Theme::Dark,
            _ => Theme::Light,
        };
        ThemeResources::new(theme, AccentPalette::default())
    }

    /// A control's own resources, as `{ThemeResource ButtonBackground}` resolves in this theme.
    pub fn button(&self) -> ButtonResources {
        ButtonResources::for_theme(self.theme, &self.accent)
    }

    /// The text editor's brushes, including its focused elevation border.
    pub fn text_box(&self) -> TextBoxResources {
        TextBoxResources::for_theme(self.theme, &self.accent)
    }

    /// The password editor's shared text-control and reveal-button brushes.
    pub fn password_box(&self) -> PasswordBoxResources {
        PasswordBoxResources::for_theme(self.theme, &self.accent)
    }

    /// ProgressBar's track and status colors.
    pub fn progress_bar(&self) -> ProgressBarResources {
        ProgressBarResources::for_theme(self.theme, &self.accent)
    }

    /// ProgressRing's arc and track colors.
    pub fn progress_ring(&self) -> ProgressRingResources {
        ProgressRingResources::for_theme(self.theme, &self.accent)
    }

    /// Expander's header, chevron and content colors.
    pub fn expander(&self) -> ExpanderResources {
        ExpanderResources::for_theme(self.theme, &self.accent)
    }

    /// InfoBadge's accent fill and the foreground on it.
    pub fn info_badge(&self) -> InfoBadgeResources {
        InfoBadgeResources::for_theme(self.theme, &self.accent)
    }

    /// InfoBar's severity and content colors.
    pub fn info_bar(&self) -> InfoBarResources {
        InfoBarResources::for_theme(self.theme, &self.accent)
    }

    pub fn toggle_switch(&self) -> ToggleSwitchResources {
        ToggleSwitchResources::for_theme(self.theme, &self.accent)
    }

    pub fn check_box(&self) -> CheckBoxResources {
        CheckBoxResources::for_theme(self.theme, &self.accent)
    }

    pub fn radio_button(&self) -> RadioButtonResources {
        RadioButtonResources::for_theme(self.theme, &self.accent)
    }

    pub fn toggle_button(&self) -> ToggleButtonResources {
        ToggleButtonResources::for_theme(self.theme, &self.accent)
    }

    pub fn hyperlink_button(&self) -> HyperlinkButtonResources {
        HyperlinkButtonResources::for_theme(self.theme, &self.accent)
    }

    pub fn repeat_button(&self) -> RepeatButtonResources {
        RepeatButtonResources::for_theme(self.theme, &self.accent)
    }

    pub fn slider(&self) -> SliderResources {
        SliderResources::for_theme(self.theme, &self.accent)
    }

    pub fn split_view(&self) -> SplitViewResources {
        SplitViewResources::for_theme(self.theme, &self.accent)
    }

    pub fn tool_tip(&self) -> ToolTipResources {
        ToolTipResources::for_theme(self.theme, &self.accent)
    }

    /// The tab strip and tab item resources for this theme.
    pub fn tab_view(&self) -> TabViewResources {
        TabViewResources::for_theme(self.theme, &self.accent)
    }

    /// The desktop scroll indicator resources for this theme.
    pub fn scroll_bar(&self) -> ScrollBarResources {
        ScrollBarResources::for_theme(self.theme, &self.accent)
    }

    /// The interactive popup presenter resources for this theme.
    pub fn flyout_presenter(&self) -> FlyoutPresenterResources {
        FlyoutPresenterResources::for_theme(self.theme, &self.accent)
    }

    /// The menu presenter and menu item resources for this theme.
    pub fn menu_flyout(&self) -> MenuFlyoutResources {
        MenuFlyoutResources::for_theme(self.theme, &self.accent)
    }

    /// The dropdown chevron resources for this theme.
    pub fn drop_down_button(&self) -> DropDownButtonResources {
        DropDownButtonResources::for_theme(self.theme, &self.accent)
    }

    /// Resolves MenuBar and MenuBarItem template resources.
    pub fn menu_bar(&self) -> MenuBarResources {
        MenuBarResources::for_theme(self.theme, &self.accent)
    }

    /// The split button resources for this theme.
    pub fn split_button(&self) -> SplitButtonResources {
        SplitButtonResources::for_theme(self.theme, &self.accent)
    }

    /// The radio menu item resources for this theme.
    pub fn radio_menu_flyout_item(&self) -> RadioMenuFlyoutItemResources {
        RadioMenuFlyoutItemResources::for_theme(self.theme, &self.accent)
    }

    /// The navigation back and close button resources for this theme.
    pub fn navigation_back_button(&self) -> NavigationBackButtonResources {
        NavigationBackButtonResources::for_theme(self.theme, &self.accent)
    }

    /// The navigation pane and top navigation resources for this theme.
    pub fn navigation_view(&self) -> NavigationViewResources {
        NavigationViewResources::for_theme(self.theme, &self.accent)
    }
}

/// Sets the theme and accent for a subtree; XAML `FrameworkElement.RequestedTheme` plus the accent the OS would supply.
#[derive(Debug)]
pub struct ThemeScope {
    pub resources: ThemeResources,
    pub child: WidgetRef,
}

impl ThemeScope {
    pub fn new<K>(theme: Theme, child: impl IntoWidget<K>) -> ThemeScope {
        ThemeScope {
            resources: ThemeResources::new(theme, AccentPalette::default()),
            child: child.into_widget(),
        }
    }

    pub fn accent(mut self, accent: AccentPalette) -> ThemeScope {
        self.resources = ThemeResources::new(self.resources.theme, accent);
        self
    }
}

impl InheritedWidget for ThemeScope {
    fn child(&self) -> &WidgetRef {
        &self.child
    }
    fn update_should_notify(&self, old: &Self) -> bool {
        self.resources != old.resources
    }
}
