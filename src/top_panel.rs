use eframe::egui;

use crate::file::{pick_file_to, save_file_to};
use crate::types::*;
use crate::MyApp;

impl MyApp {
    pub(crate) fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("my_top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut save_path = None;
                let mut open_path = None;
                if ui.small_button("Save").on_hover_text("Ctrl + S").clicked() {
                    save_file_to(&mut save_path, ("Level", &["lvl"]));
                    if let Some(path) = save_path {
                        self.save(path);
                    }
                }
                if ui.small_button("Open").on_hover_text("Ctrl + O").clicked() {
                    pick_file_to(&mut open_path, ("Level", &["lvl"]));
                    if let Some(path) = open_path {
                        self.open(path);
                    }
                }
                ui.separator();
                if ui.small_button("Clear").clicked() {
                    self.show_clear_confirmation = true;
                }
                ui.separator();
                ui.radio_value(&mut self.current_mode, Mode::DrawBackground, "Background")
                    .on_hover_text("M");
                ui.radio_value(&mut self.current_mode, Mode::DrawForeground, "Foreground")
                    .on_hover_text("M");
                ui.radio_value(&mut self.current_mode, Mode::Collision, "Collision")
                    .on_hover_text("M");
                ui.radio_value(&mut self.current_mode, Mode::Entity, "Entity")
                    .on_hover_text("M");
                ui.separator();
                ui.label("View Filter");
                ui.checkbox(&mut self.show_foreground, "Foreground");
                ui.checkbox(&mut self.show_background, "Background");
                ui.checkbox(&mut self.show_collision, "Collision");
                ui.checkbox(&mut self.show_entity, "Entity");
                ui.checkbox(&mut self.show_grid, "Grid");
            });
        });
    }
}
