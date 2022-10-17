use std::path::PathBuf;

use eframe::egui;
use egui::{Pos2, Rect, Vec2};

use crate::file::pick_file_to;
use crate::types::*;
use crate::MyApp;

impl MyApp {
    fn side_panel_settings(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Settings", |ui| {
            ui.label("Sprite Size");
            ui.add(
                egui::DragValue::new(&mut self.spritesheet_info.sprite_size)
                    .clamp_range(1..=1024)
                    .suffix("px"),
            );
            ui.label("Num Rows");
            ui.add(egui::DragValue::new(&mut self.spritesheet_info.num_rows).clamp_range(1..=255));
            ui.label("Num Columns");
            ui.add(egui::DragValue::new(&mut self.spritesheet_info.num_cols).clamp_range(1..=255));
            ui.separator();
            ui.label("Preview Display");
            ui.radio_value(
                &mut self.spritesheet_col_orientation,
                ColumnOrientation::Minor,
                "Row Major",
            );
            ui.radio_value(
                &mut self.spritesheet_col_orientation,
                ColumnOrientation::Major,
                "Column Major",
            );
        });
    }
    fn side_panel_sprite_selector_make_img_btn(
        &self,
        x: u32,
        y: u32,
        handle: &egui::TextureHandle,
    ) -> (egui::ImageButton, Rect) {
        let handle_size = handle.size_vec2();
        let normalized_x = x as f32 / handle_size.x;
        let normalized_y = y as f32 / handle_size.y;
        let max_x =
            (x as f32 + handle_size.x / self.spritesheet_info.num_rows as f32) / handle_size.x;
        let max_y =
            (y as f32 + handle_size.y / self.spritesheet_info.num_cols as f32) / handle_size.y;
        let uv = Rect {
            min: Pos2 {
                x: normalized_x,
                y: normalized_y,
            },
            max: Pos2 { x: max_x, y: max_y },
        };
        let mut img_btn = egui::widgets::ImageButton::new(
            handle,
            Vec2 {
                x: handle_size.x / self.spritesheet_info.num_rows as f32,
                y: handle_size.y / self.spritesheet_info.num_cols as f32,
            },
        )
        .uv(uv);
        if self.selected_uv == Some(uv) {
            img_btn = img_btn.selected(true);
        }
        (img_btn, uv)
    }
    fn side_panel_sprite_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if let Some(handle) = &self.spritesheet_handle {
                let handle_size = handle.size_vec2();
                if matches!(self.spritesheet_col_orientation, ColumnOrientation::Major) {
                    for x in (0..handle_size.x as u32)
                        .step_by((handle_size.x / self.spritesheet_info.num_rows as f32) as usize)
                    {
                        for y in (0..handle_size.y as u32).step_by(
                            (handle_size.y / self.spritesheet_info.num_cols as f32) as usize,
                        ) {
                            let (img_btn, uv) =
                                self.side_panel_sprite_selector_make_img_btn(x, y, handle);
                            if ui.add(img_btn).clicked() {
                                self.selected_uv = Some(uv);
                            };
                        }
                    }
                } else {
                    for y in (0..handle_size.y as u32)
                        .step_by((handle_size.y / self.spritesheet_info.num_cols as f32) as usize)
                    {
                        for x in (0..handle_size.x as u32).step_by(
                            (handle_size.x / self.spritesheet_info.num_rows as f32) as usize,
                        ) {
                            let (img_btn, uv) =
                                self.side_panel_sprite_selector_make_img_btn(x, y, handle);
                            if ui.add(img_btn).clicked() {
                                self.selected_uv = Some(uv);
                            };
                        }
                    }
                }
            }
        });
    }
    fn side_panel_spritesheet_preview(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        spritesheet_path: Option<PathBuf>,
    ) {
        if let Some(path) = spritesheet_path {
            let image_result = image::io::Reader::open(path).unwrap().decode();
            if image_result.is_err() {
                return;
            }
            let image = image_result.unwrap();
            let size = [image.width() as usize, image.height() as usize];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            self.spritesheet_handle =
                Some(ctx.load_texture("example-image", color_image, egui::TextureFilter::Nearest));
        }
        if let Some(handle) = &self.spritesheet_handle {
            ui.image(handle, handle.size_vec2());
        }
    }
    pub(crate) fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("my_right_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut spritesheet_path = None;
                if ui.button("Open Spritesheet").clicked() {
                    pick_file_to(
                        &mut spritesheet_path,
                        ("image", &["webp", "png", "bmp", "jpg", "jpeg"]),
                    );
                }
                self.side_panel_spritesheet_preview(ctx, ui, spritesheet_path);
                self.side_panel_settings(ui);
                ui.separator();
                self.side_panel_sprite_selector(ui);
            });
        });
    }
}
