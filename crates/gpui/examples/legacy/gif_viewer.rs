<<<<<<< HEAD:crates/gpui/examples/legacy/gif_viewer.rs
=======
#![cfg_attr(target_family = "wasm", no_main)]

#[path = "example_support/fonts.rs"]
mod example_support;

>>>>>>> ae625934ba7c510bdf18099911e025fc9bee4e57:crates/gpui/examples/gif_viewer.rs
use gpui::{App, Context, Render, Window, WindowOptions, div, img, prelude::*};
use std::path::PathBuf;

struct GifViewer {
    gif_path: PathBuf,
}

impl GifViewer {
    fn new(gif_path: PathBuf) -> Self {
        Self { gif_path }
    }
}

impl Render for GifViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            img(self.gif_path.clone())
                .size_full()
                .object_fit(gpui::ObjectFit::Contain)
                .id("gif"),
        )
    }
}

<<<<<<< HEAD:crates/gpui/examples/legacy/gif_viewer.rs
fn main() {
    env_logger::init();
    gpui_platform::application().run(|cx: &mut App| {
        let gif_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/legacy/image/black-cat-typing.gif");
=======
fn run_example() {
    application().run(|cx: &mut App| {
        if !example_support::load_fonts(cx) {
            return;
        }
        let gif_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/image/black-cat-typing.gif");
>>>>>>> ae625934ba7c510bdf18099911e025fc9bee4e57:crates/gpui/examples/gif_viewer.rs

        cx.open_window(
            WindowOptions {
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(|_| GifViewer::new(gif_path)),
        )
        .unwrap();
        cx.activate(true);
    });
}
