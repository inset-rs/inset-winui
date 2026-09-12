//! Gallery setup and page navigation layered on reusable test support.
#![allow(dead_code, unused_imports)]

pub use inset_winui_test_support::{Fixture, mount};

/// Gallery-only entry points; the shared fixture has no dependency on gallery code.
pub trait GalleryFixtureExt: Sized {
    /// Mounts the gallery's initial page.
    fn new(size: [u32; 2]) -> Self;

    /// Mounts the gallery directly on one feature.
    fn for_feature(size: [u32; 2], feature: winui_gallery::Feature) -> Self;

    /// Navigates through the gallery's visible navigation item.
    fn navigate(&mut self, feature: winui_gallery::Feature);
}

impl GalleryFixtureExt for Fixture {
    fn new(size: [u32; 2]) -> Self {
        Self::with_root(size, winui_gallery::run)
    }

    fn for_feature(size: [u32; 2], feature: winui_gallery::Feature) -> Self {
        Self::with_root(size, move |app| winui_gallery::run_feature(app, feature))
    }

    fn navigate(&mut self, feature: winui_gallery::Feature) {
        self.ensure_visible(feature.title());
        self.tap(feature.title());
        self.pump();
    }
}
