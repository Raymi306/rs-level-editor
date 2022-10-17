use eframe::egui;
use egui::{Rect, Vec2};
use std::collections::HashMap;

use crate::types::*;
use crate::MyApp;

impl MyApp {
    fn handle_plot_clicks(
        &mut self,
        plot_ui: &egui::plot::PlotUi,
        primary_clicked: bool,
        secondary_clicked: bool,
        is_drag: bool,
    ) {
        if !(primary_clicked || secondary_clicked || is_drag) {
            return;
        }
        if let Some(coord) = plot_ui.pointer_coordinate() {
            let coord_x = coord.x.floor();
            let coord_y = coord.y.floor();
            let point = egui::widgets::plot::PlotPoint {
                x: coord_x,
                y: coord_y,
            };
            let plot_bounds = plot_ui.plot_bounds();
            let min = plot_bounds.min();
            let max = plot_bounds.max();
            let hashable_point = HashableVec2::from(point);
            if !(coord.x < min[0] || coord.x > max[0] || coord.y < min[1] || coord.y > max[1])
                && !self.show_clear_confirmation
                && !self.show_entity_popup
            // stop when pop ups are open
            {
                match self.current_mode {
                    Mode::DrawForeground | Mode::DrawBackground => {
                        self.handle_plot_fg_bg_clicks(
                            primary_clicked,
                            secondary_clicked,
                            is_drag,
                            hashable_point,
                        );
                    }
                    Mode::Collision => {
                        self.handle_plot_collision_clicks(primary_clicked, is_drag, hashable_point);
                    }
                    Mode::Entity => {
                        self.handle_plot_entity_clicks(
                            primary_clicked,
                            secondary_clicked,
                            is_drag,
                            hashable_point,
                        );
                    }
                }
            }
        }
    }
    fn handle_plot_fg_bg_clicks(
        &mut self,
        primary_clicked: bool,
        secondary_clicked: bool,
        is_drag: bool,
        hashable_point: HashableVec2,
    ) {
        if let Some(selected_uv) = self.selected_uv {
            let (layer_plotted_tiles, mut action) = match self.current_mode {
                Mode::DrawBackground => (
                    &mut self.background_plotted_tiles,
                    Action::ClickBackground(hashable_point, selected_uv, None, is_drag),
                ),
                Mode::DrawForeground => (
                    &mut self.foreground_plotted_tiles,
                    Action::ClickForeground(hashable_point, selected_uv, None, is_drag),
                ),
                _ => unreachable!(),
            };
            if primary_clicked || is_drag {
                if !is_drag {
                    if let Some(original_uv) = layer_plotted_tiles.remove(&hashable_point) {
                        action = match self.current_mode {
                            Mode::DrawBackground => Action::ClickBackground(
                                hashable_point,
                                selected_uv,
                                Some(original_uv),
                                is_drag,
                            ),
                            Mode::DrawForeground => Action::ClickForeground(
                                hashable_point,
                                selected_uv,
                                Some(original_uv),
                                is_drag,
                            ),
                            _ => unreachable!(),
                        };
                    } else {
                        layer_plotted_tiles.insert(hashable_point, selected_uv);
                    }
                    self.undo_queue.push(action);
                    self.redo_queue.clear();
                } else {
                    if let Some(original_uv) =
                        layer_plotted_tiles.insert(hashable_point, selected_uv)
                    {
                        if original_uv != selected_uv {
                            action = match self.current_mode {
                                Mode::DrawBackground => Action::ClickBackground(
                                    hashable_point,
                                    selected_uv,
                                    Some(original_uv),
                                    is_drag,
                                ),
                                Mode::DrawForeground => Action::ClickForeground(
                                    hashable_point,
                                    selected_uv,
                                    Some(original_uv),
                                    is_drag,
                                ),
                                _ => unreachable!(),
                            };
                            self.undo_queue.push(action);
                            self.redo_queue.clear();
                        }
                    } else {
                        self.undo_queue.push(action);
                        self.redo_queue.clear();
                    }
                }
            }
        }
        if secondary_clicked {
            match self.current_mode {
                Mode::DrawBackground => {
                    if let Some(uv) = self.background_plotted_tiles.get(&hashable_point) {
                        self.selected_uv = Some(*uv);
                    }
                }
                Mode::DrawForeground => {
                    if let Some(uv) = self.foreground_plotted_tiles.get(&hashable_point) {
                        self.selected_uv = Some(*uv);
                    }
                }
                _ => unreachable!(),
            };
        }
    }
    fn handle_plot_collision_clicks(
        &mut self,
        primary_clicked: bool,
        is_drag: bool,
        hashable_point: HashableVec2,
    ) {
        if primary_clicked || is_drag {
            if !is_drag {
                if !self.collision_tiles.remove(&hashable_point) {
                    self.collision_tiles.insert(hashable_point);
                }
                self.undo_queue.push(Action::ClickCollision(hashable_point));
                self.redo_queue.clear();
            } else {
                if self.collision_tiles.insert(hashable_point) {
                    self.undo_queue.push(Action::ClickCollision(hashable_point));
                    self.redo_queue.clear();
                }
            }
        }
    }
    fn handle_plot_entity_clicks(
        &mut self,
        primary_clicked: bool,
        secondary_clicked: bool,
        is_drag: bool,
        hashable_point: HashableVec2,
    ) {
        if primary_clicked || is_drag {
            if !is_drag {
                let mut action_label = None;
                if let Some(label) = self.entity_tiles.remove(&hashable_point) {
                    action_label = Some(label.clone());
                    self.entity_descriptions.remove(&label);
                } else {
                    self.entity_tiles.insert(hashable_point, "".to_owned());
                    self.show_entity_popup = true;
                    self.prev_entity_description = "".to_owned();
                    self.selected_entity = Some(hashable_point);
                }
                self.undo_queue
                    .push(Action::ClickEntity(hashable_point, action_label));
                self.redo_queue.clear();
            }
        } else if secondary_clicked {
            if let Some(description) = self.entity_tiles.get(&hashable_point) {
                self.show_entity_popup = true;
                self.entity_description = description.clone();
                self.prev_entity_description = self.entity_description.clone();
                self.selected_entity = Some(hashable_point);
            }
        }
    }
    fn draw_sprites_on_plot(
        &self,
        plot_ui: &mut egui::plot::PlotUi,
        handle: &egui::TextureHandle,
        plotted_tiles: &HashMap<HashableVec2, Rect>,
    ) {
        let handle_size = handle.size_vec2();
        for (point, uv) in plotted_tiles {
            let final_coord = egui::widgets::plot::PlotPoint {
                x: point.x as f64 + 0.5,
                y: point.y as f64 + 0.5,
            };
            let img = egui::widgets::plot::PlotImage::new(
                handle,
                final_coord,
                Vec2 {
                    x: handle_size.x / self.spritesheet_info.num_rows as f32,
                    y: handle_size.y / self.spritesheet_info.num_cols as f32,
                } / self.spritesheet_info.sprite_size as f32,
            )
            .uv(*uv);
            plot_ui.image(img);
        }
    }
    fn draw_on_plot(&mut self, plot_ui: &mut egui::plot::PlotUi) {
        // if we want to draw sprites, we need a spritesheet
        if let Some(handle) = &self.spritesheet_handle {
            if self.show_background {
                self.draw_sprites_on_plot(plot_ui, handle, &self.background_plotted_tiles);
            }
            if self.show_foreground {
                self.draw_sprites_on_plot(plot_ui, handle, &self.foreground_plotted_tiles);
            }
        }
        // can draw these without spritesheet
        if self.show_collision {
            let collision_plot_points: Vec<[f64; 2]> = self
                .collision_tiles
                .iter()
                .map(|point| [point.x as f64 + 0.5, point.y as f64 + 0.5])
                .collect();
            let collision_points = egui::plot::Points::new(collision_plot_points)
                .filled(false)
                .radius(10.0)
                .shape(egui::plot::MarkerShape::Square)
                .color(egui::Color32::from_rgb(255, 0, 0));
            plot_ui.points(collision_points);
        }
        if self.show_entity {
            let entity_plot_points: Vec<[f64; 2]> = self
                .entity_tiles
                .iter()
                .map(|(point, _)| [point.x as f64 + 0.5, point.y as f64 + 0.5])
                .collect();
            let entity_points = egui::plot::Points::new(entity_plot_points)
                .filled(false)
                .radius(10.0)
                .shape(egui::plot::MarkerShape::Diamond)
                .color(egui::Color32::from_rgb(0, 255, 255));
            plot_ui.points(entity_points);
        }
    }
    pub(crate) fn plot_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut plot = egui::plot::Plot::new("level_plot")
                .data_aspect(1.0)
                .x_grid_spacer(egui::widgets::plot::uniform_grid_spacer(|_| {
                    [100.0, 25.0, 1.0]
                }))
                .y_grid_spacer(egui::widgets::plot::uniform_grid_spacer(|_| {
                    [100.0, 25.0, 1.0]
                }))
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_double_click_reset(false);
            if !self.show_grid {
                plot = plot.show_axes([false, false]);
            }
            plot.show(ui, |plot_ui| {
                let ctx = plot_ui.ctx();
                //plot_ui.translate(Vec2 {x: 1.0, y: 0.0}); # TODO add alternative means of
                //navigating plot
                let mut primary_clicked = false;
                let mut secondary_clicked = false;
                for event in &ctx.input().events {
                    if let egui::Event::PointerButton {
                        button, pressed, ..
                    } = event
                    {
                        if *pressed {
                            match button {
                                egui::PointerButton::Primary => primary_clicked = true,
                                egui::PointerButton::Secondary => secondary_clicked = true,
                                _ => (),
                            }
                        }
                    }
                }
                let drag_delta = plot_ui.pointer_coordinate_drag_delta();
                let is_drag;

                // attempting to stop user from clicking with a slight drag on mouse getting
                // detected as a drag within a single square, thus performing 2 actions at once
                // TODO a better alternative would be to add the delta with the position and see if
                // it steps into another tile
                if !(drag_delta.x > -0.05 && drag_delta.x < 0.05)
                    || !(drag_delta.y > -0.05 && drag_delta.y < 0.05)
                {
                    is_drag = true;
                } else {
                    is_drag = false;
                }
                self.handle_plot_clicks(plot_ui, primary_clicked, secondary_clicked, is_drag);
                // Draw sprites, background then foreground
                self.draw_on_plot(plot_ui);
            });
        });
    }
}
