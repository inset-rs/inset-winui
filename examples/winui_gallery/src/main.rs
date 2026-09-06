use reveal_embedder_winit::{ImplicitViewConfig, WinitEmbedder};
use reveal_shell::Shell;
fn main() {
    WinitEmbedder {
        implicit_view: Some(ImplicitViewConfig {
            title: "WinUI Gallery".into(),
            logical_size: [1080.0, 780.0],
        }),
    }
    .run(|platform| Shell::new(platform, winui_gallery::run));
}
