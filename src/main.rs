mod settings;

slint::include_modules!();

// use std::{
    // fs::File,
    // io::{BufReader, Cursor, Read},
    // rc::Rc,
    // sync::mpsc
// };
use anyhow::Result;
// use hex_color::HexColor;
// use serde::{Serialize, Deserialize};
use single_instance::SingleInstance;

pub const PROGRESS_BYTES: &str = include_str!("../assets/Circle.svg");

// fn color_to_hex(color: slint::Color) -> String {
//     format!(
//         "#{:02X}{:02X}{:02X}",
//         color.red(),
//         color.green(),
//         color.blue()
//     )
// }

// fn update_progress_svg(remaining_period: f32) -> slint::Image {
//     slint::Image::load_from_svg_data(
//         PROGRESS_BYTES.replace(
//             "stroke-dasharray=\"100, 100\"",
//             &format!("stroke-dasharray=\"{}, 100\"", remaining_period)
//         )
//         .as_bytes()
//     )
//     .unwrap()
// }

fn main() -> Result<(), anyhow::Error> {

    let instance = SingleInstance::new("io.github.risuleia.tranquilo").unwrap();
    if !instance.is_single() {
        return Err(anyhow::anyhow!("One instance of Tranquilo is already running."));
    }

    let ui = AppWindow::new()?;

    let close_handle = ui.as_weak();
    ui.on_close_window(move || {
        let close_handle = close_handle.upgrade().unwrap();
        close_handle.hide().unwrap();
    });

    let minimize_handle = ui.as_weak();
    ui.on_minimize_window(move || {
        let minimize_handle = minimize_handle.upgrade().unwrap();
        i_slint_backend_winit::WinitWindowAccessor::with_winit_window(
            minimize_handle.window(),
            |window| window.set_minimized(true)
        );
    });

    let move_hande = ui.as_weak();
    ui.on_move_window(move || {
        let move_handle = move_hande.upgrade().unwrap();
        i_slint_backend_winit::WinitWindowAccessor::with_winit_window(
            move_handle.window(),
            |window| window.drag_window()
        );
    });
    
    let _ = ui.run();
    Ok(())
}