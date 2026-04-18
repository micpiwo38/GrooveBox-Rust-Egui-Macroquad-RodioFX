
use egui_macroquad::egui::{panel, TopBottomPanel, CentralPanel};
use egui_macroquad::egui::Context;
use crate::app_state::AppState;

pub fn draw(ctx: &Context, state: &mut AppState){
    TopBottomPanel::bottom("bottom_panel")
        .resizable(false)
        .default_height(400.0)
        .show(ctx, |ui| {
            crate::gui::bottom_panel::draw(ui, state);
        });
    CentralPanel::default()
        .show(ctx, |ui| {
            crate::gui::top_panel::draw(ui, state);
        });
}
