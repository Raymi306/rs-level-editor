use std::collections::HashMap;
use std::path::PathBuf;

use eframe;
use eframe::egui::{Vec2, Rect, Pos2};
use eframe::egui;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct HashableVec2 {
    x: i64,
    y: i64,
}

impl From<egui::widgets::plot::PlotPoint> for HashableVec2 {
    fn from(point: egui::widgets::plot::PlotPoint) -> HashableVec2 {
        HashableVec2 {
            x: point.x as i64,
            y: point.y as i64,
        }
    }
}

impl From<HashableVec2> for egui::widgets::plot::PlotPoint {
    fn from(point: HashableVec2) -> egui::widgets::plot::PlotPoint {
        egui::widgets::plot::PlotPoint {
            x: point.x as f64,
            y: point.y as f64,
        }
    }
}

struct MyApp {
    save_path: Option<PathBuf>,
    open_path: Option<PathBuf>,
    open_spritesheet_path: Option<PathBuf>,
    spritesheet_handle: Option<egui::TextureHandle>,
    plotted_tiles: HashMap<HashableVec2, Rect>,
    selected_uv: Option<Rect>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            save_path: None,
            open_path: None,
            open_spritesheet_path: None,
            spritesheet_handle: None,
            plotted_tiles: HashMap::new(),
            selected_uv: None,
        }
    }
}

fn pick_file_to(var: &mut Option<PathBuf>) {
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        *var = Some(path);
        println!("{:?}", var);
    }
}

impl MyApp {

    fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("my_top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.small_button("Save").on_hover_text("Ctrl + S").clicked() {
                    pick_file_to(&mut self.save_path);
                }
                if ui.small_button("Open").on_hover_text("Ctrl + O").clicked() {
                    pick_file_to(&mut self.open_path);
                }
            });
        });
    }

    fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("my_right_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if ui.button("Open Spritesheet").clicked() {
                    pick_file_to(&mut self.open_spritesheet_path);
                }
                if let Some(path) = &self.open_spritesheet_path {
                    let image = image::io::Reader::open(path).unwrap().decode().unwrap();
                    let size = [image.width() as usize, image.height() as usize];
                    let image_buffer = image.to_rgba8();
                    let pixels = image_buffer.as_flat_samples();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice(),);
                    self.spritesheet_handle = Some(ctx.load_texture(
                        "example-image",
                        color_image,
                        egui::TextureFilter::Nearest
                    ));
                } 
                if let Some(handle) = &self.spritesheet_handle {
                    ui.image(handle, handle.size_vec2());
                }
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    if let Some(handle) = &self.spritesheet_handle {
                        //println!("{:?}", handle.size_vec2()); // 320, 320
                        let handle_size = handle.size_vec2();
                        for x in (0..handle_size.x as u32).step_by((handle_size.x / 10.0) as usize) {
                            for y in (0..handle_size.y as u32).step_by((handle_size.y / 10.0) as usize) {
                                let normalized_x = x as f32 / handle_size.x;
                                let normalized_y = y as f32 / handle_size.y;
                                let max_x = (x as f32 + handle_size.x / 10.0) / handle_size.x;
                                let max_y = (y as f32 + handle_size.y / 10.0) / handle_size.y;
                                let uv = Rect {
                                    min: Pos2 { x: normalized_x, y: normalized_y },
                                    max: Pos2 { x: max_x, y: max_y },
                                };
                                let mut img_btn = egui::widgets::ImageButton::new(
                                    handle, Vec2 {x: handle_size.x / 10.0, y: handle_size.y / 10.0}
                                ).uv(uv);
                                if self.selected_uv == Some(uv) {
                                    img_btn = img_btn.selected(true);
                                }
                                if ui.add(img_btn).clicked() {
                                    self.selected_uv = Some(uv);
                                };
                            }
                        }
                    }
                });
            });
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_panel(ctx);
        self.side_panel(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::plot::Plot::new("level_plot")
                .data_aspect(1.0)
                .x_grid_spacer(egui::widgets::plot::uniform_grid_spacer(|_| [100.0, 25.0, 1.0]))
                .y_grid_spacer(egui::widgets::plot::uniform_grid_spacer(|_| [100.0, 25.0, 1.0]))
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_double_click_reset(false)
                .show(ui, |plot_ui| {
                    let mut drag_delta = plot_ui.pointer_coordinate_drag_delta();
                    // float comparisons suck
                    if drag_delta.x == -0.0 {
                        drag_delta.x = 0.0;
                    }
                    if drag_delta.y == -0.0 {
                        drag_delta.y = 0.0;
                    }
                    let is_drag = drag_delta.x != 0.0 || drag_delta.y != 0.0;
                    if plot_ui.plot_clicked() || is_drag {
                        if let (Some(coord), Some(selected_uv)) = (plot_ui.pointer_coordinate(), self.selected_uv) {
                            let coord_x = coord.x.floor();
                            let coord_y = coord.y.floor();
                            let point = egui::widgets::plot::PlotPoint {x: coord_x, y: coord_y};
                            if !is_drag {
                                if let None = self.plotted_tiles.remove(&HashableVec2::from(point)) {
                                    self.plotted_tiles.insert(HashableVec2::from(point), selected_uv);
                                }
                            } else {
                                self.plotted_tiles.insert(HashableVec2::from(point), selected_uv);
                            }
                        }
                    }
                    if let Some(handle) = &self.spritesheet_handle {
                        for (point, uv) in &self.plotted_tiles {
                            let handle_size = handle.size_vec2();
                            let final_coord = egui::widgets::plot::PlotPoint {
                                x: point.x as f64 + 0.5,
                                y: point.y as f64 + 0.5,
                            };
                            let img = egui::widgets::plot::PlotImage::new(handle, final_coord, Vec2 {x: handle_size.x / 10.0, y: handle_size.y / 10.0} / 32.0)
                                .uv(*uv);
                            plot_ui.image(img);
                        }
                    }
                });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Level Editor",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}
