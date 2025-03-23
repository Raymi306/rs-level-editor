use eframe::egui;

use crate::types::*;
use crate::MyApp;

impl MyApp {
    pub(crate) fn handle_clear_confirmation_popup(&mut self, ctx: &egui::Context) {
        if self.show_clear_confirmation {
            egui::Window::new("Are you sure you want to erase everything? This cannot be undone")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_clear_confirmation = false;
                        }
                        if ui.button("Ok").clicked() {
                            self.clear();
                        }
                    });
                });
        }
    }
    // TODO maybe pass the entity key in rather than getting it inside
    fn entity_description_is_ok(&self) -> (bool, HashableVec2) {
        if let Some(entity_key) = self.selected_entity {
            return (
                !self.entity_descriptions.contains(&self.entity_description)
                    && !self.entity_description.is_empty(),
                entity_key,
            );
        }
        unreachable!(); // boy I hope so
    }
    fn do_entity_ok(&mut self, entity_key: HashableVec2) {
        if self.prev_entity_description != self.entity_description {
            self.entity_descriptions
                .remove(&self.prev_entity_description);
        }
        self.show_entity_popup = false;
        self.entity_tiles
            .insert(entity_key, self.entity_description.clone());
        self.entity_descriptions
            .insert(self.entity_description.clone());
        self.entity_description = "".to_string();
        self.redo_queue.clear(); // very unhappy with the state of undo/redo for entity editing but
    }
    pub(crate) fn handle_entity_popup(&mut self, ctx: &egui::Context) {
        if self.show_entity_popup {
            egui::Window::new("Entity Label Editor")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    let response = ui.add(egui::TextEdit::singleline(&mut self.entity_description));
                    self.entity_description = self.entity_description.trim().to_string();
                    if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                        let (description_is_ok, entity_key) = self.entity_description_is_ok();
                        if description_is_ok {
                            self.do_entity_ok(entity_key);
                        }
                    }
                    response.request_focus();
                    ui.label("Existing Entities");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for label in &self.entity_descriptions {
                                ui.label(label);
                            }
                        });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            let (description_is_ok, entity_key) = self.entity_description_is_ok();
                            if (!description_is_ok) || self.prev_entity_description.is_empty() {
                                self.entity_tiles.remove(&entity_key);
                            }
                            self.show_entity_popup = false;
                            self.entity_description = "".to_string();
                        }
                        let (description_is_ok, entity_key) = self.entity_description_is_ok();
                        if description_is_ok {
                            if ui.button("Ok").clicked() {
                                self.do_entity_ok(entity_key);
                            }
                        } else {
                            ui.label("You must choose a unique label");
                        }
                    });
                });
        }
    }
}
