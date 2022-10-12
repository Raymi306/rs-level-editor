use std::collections::{HashMap, HashSet, BTreeSet};
use std::path::PathBuf;

use eframe;
use eframe::egui;
use eframe::egui::{Pos2, Rect, Vec2};

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

struct SpritesheetInfo {
    sprite_size: u16,
    num_rows: u8,
    num_cols: u8,
}

impl Default for SpritesheetInfo {
    fn default() -> Self {
        Self {
            sprite_size: 32,
            num_rows: 10,
            num_cols: 10,
        }
    }
}

#[derive(PartialEq, Debug)]
enum Mode {
    DrawBackground,
    DrawForeground,
    Collision,
    Entity,
}

struct MyApp {
    spritesheet_info: SpritesheetInfo,
    spritesheet_handle: Option<egui::TextureHandle>,
    foreground_plotted_tiles: HashMap<HashableVec2, Rect>,
    background_plotted_tiles: HashMap<HashableVec2, Rect>,
    collision_tiles: HashSet<HashableVec2>,
    entity_tiles: HashMap<HashableVec2, String>,
    selected_uv: Option<Rect>,
    selected_entity: Option<HashableVec2>,
    entity_description: String,
    prev_entity_description: String,
    entity_descriptions: BTreeSet<String>,
    current_mode: Mode,
    show_entity_popup: bool,
    show_clear_confirmation: bool,
    show_foreground: bool,
    show_background: bool,
    show_collision: bool,
    show_entity: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            spritesheet_info: SpritesheetInfo::default(),
            spritesheet_handle: None,
            foreground_plotted_tiles: HashMap::new(),
            background_plotted_tiles: HashMap::new(),
            collision_tiles: HashSet::new(),
            entity_tiles: HashMap::new(),
            selected_uv: None,
            selected_entity: None,
            entity_description: "".to_string(),
            prev_entity_description: "".to_string(),
            entity_descriptions: BTreeSet::new(),
            current_mode: Mode::DrawBackground,
            show_entity_popup: false,
            show_clear_confirmation: false,
            show_foreground: true,
            show_background: true,
            show_collision: true,
            show_entity: true,
        }
    }
}

fn pick_file_to(var: &mut Option<PathBuf>, filter: (&str, &[&str])) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(filter.0, filter.1)
        .pick_file()
    {
        *var = Some(path);
        println!("{:?}", var);
    }
}

impl MyApp {
    fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("my_top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut save_path = None;
                let mut open_path = None;
                if ui.small_button("Save").on_hover_text("Ctrl + S").clicked() {
                    pick_file_to(&mut save_path, ("Level", &["lvl"]));
                }
                if ui.small_button("Open").on_hover_text("Ctrl + O").clicked() {
                    pick_file_to(&mut open_path, ("Level", &["lvl"]));
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
            });
        });
    }

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
        });
    }

    fn side_panel_sprite_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if let Some(handle) = &self.spritesheet_handle {
                //println!("{:?}", handle.size_vec2()); // 320, 320
                let handle_size = handle.size_vec2();
                for x in (0..handle_size.x as u32)
                    .step_by((handle_size.x / self.spritesheet_info.num_rows as f32) as usize)
                {
                    for y in (0..handle_size.y as u32)
                        .step_by((handle_size.y / self.spritesheet_info.num_cols as f32) as usize)
                    {
                        let normalized_x = x as f32 / handle_size.x;
                        let normalized_y = y as f32 / handle_size.y;
                        let max_x = (x as f32
                            + handle_size.x / self.spritesheet_info.num_rows as f32)
                            / handle_size.x;
                        let max_y = (y as f32
                            + handle_size.y / self.spritesheet_info.num_cols as f32)
                            / handle_size.y;
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
                        if ui.add(img_btn).clicked() {
                            self.selected_uv = Some(uv);
                        };
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
    fn side_panel(&mut self, ctx: &egui::Context) {
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
    fn plot_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::plot::Plot::new("level_plot")
                .data_aspect(1.0)
                .x_grid_spacer(egui::widgets::plot::uniform_grid_spacer(|_| {
                    [100.0, 25.0, 1.0]
                }))
                .y_grid_spacer(egui::widgets::plot::uniform_grid_spacer(|_| {
                    [100.0, 25.0, 1.0]
                }))
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_double_click_reset(false)
                .show(ui, |plot_ui| {
                    let ctx = plot_ui.ctx();
                    let mut primary_clicked = false;
                    let mut secondary_clicked = false;
                    for event in &ctx.input().raw.events {
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
                    if primary_clicked || secondary_clicked || is_drag {
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
                            if !(coord.x < min[0]
                                || coord.x > max[0]
                                || coord.y < min[1]
                                || coord.y > max[1])
                                && !self.show_clear_confirmation
                                && !self.show_entity_popup
                            // stop when pop ups are open
                            {
                                match self.current_mode {
                                    Mode::DrawForeground | Mode::DrawBackground => {
                                        let layer_plotted_tiles = match self.current_mode {
                                            Mode::DrawForeground => &mut self.foreground_plotted_tiles,
                                            Mode::DrawBackground => &mut self.background_plotted_tiles,
                                            _ => unreachable!(),
                                        };
                                        if let Some(selected_uv) = self.selected_uv {
                                            if primary_clicked || is_drag {
                                                if !is_drag {
                                                    if let None =
                                                        layer_plotted_tiles.remove(&hashable_point)
                                                    {
                                                        layer_plotted_tiles
                                                            .insert(hashable_point, selected_uv);
                                                    }
                                                } else {
                                                    layer_plotted_tiles
                                                        .insert(hashable_point, selected_uv);
                                                }
                                            } else if secondary_clicked {
                                                if let Some(uv) =
                                                    layer_plotted_tiles.get(&hashable_point)
                                                {
                                                    self.selected_uv = Some(*uv);
                                                }
                                            }
                                        }
                                    }
                                    Mode::Collision => {
                                        if primary_clicked || is_drag {
                                            if !is_drag {
                                                if !self.collision_tiles.remove(&hashable_point) {
                                                    self.collision_tiles.insert(hashable_point);
                                                }
                                            } else {
                                                self.collision_tiles.insert(hashable_point);
                                            }
                                        }
                                    }
                                    Mode::Entity => {
                                        if primary_clicked || is_drag {
                                            if !is_drag {
                                                if let Some(label) = self.entity_tiles.remove(&hashable_point) {
                                                    println!("REMOVING LABEL: {}", label);
                                                    self.entity_descriptions.remove(&label);
                                                } else {
                                                    self.entity_tiles.insert(hashable_point, "".to_owned());
                                                    self.show_entity_popup = true;
                                                    self.prev_entity_description = "".to_owned();
                                                    self.selected_entity = Some(hashable_point);
                                                }
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
                                }
                            }
                        }
                    }
                    // Draw sprites, background then foreground
                    if let Some(handle) = &self.spritesheet_handle {
                        let handle_size = handle.size_vec2();
                        if self.show_background {
                            for (point, uv) in &self.background_plotted_tiles {
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
                        if self.show_foreground {
                            for (point, uv) in &self.foreground_plotted_tiles {
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
                    }
                    // can draw these even without spritesheet
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
                    if self.show_collision {
                        plot_ui.points(collision_points);
                    }
                    if self.show_entity {
                        plot_ui.points(entity_points);
                    }

                });
        });
    }
    fn toggle_current_mode(&mut self) {
        self.current_mode = match self.current_mode {
            Mode::DrawBackground => Mode::DrawForeground,
            Mode::DrawForeground => Mode::Collision,
            Mode::Collision => Mode::Entity,
            Mode::Entity => Mode::DrawBackground,
        }
    }
    fn handle_toplevel_input(&mut self, ctx: &egui::Context) {
        if self.show_clear_confirmation || self.show_entity_popup {
            return;
        }
        for event in &ctx.input().raw.events {
            if let egui::Event::Key {
                key,
                pressed,
                modifiers: _,
            } = event
            {
                if !*pressed {
                    match key {
                        egui::Key::M => self.toggle_current_mode(),
                        _ => (),
                    };
                }
            }
        }
    }
    fn handle_clear_confirmation(&mut self, ctx: &egui::Context) {
        if self.show_clear_confirmation {
            egui::Window::new("Are you sure you want to erase everything?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_clear_confirmation = false;
                        }
                        if ui.button("Ok").clicked() {
                            self.show_clear_confirmation = false;
                            self.foreground_plotted_tiles.clear();
                            self.background_plotted_tiles.clear();
                        }
                    });
                });
        }
    }
    // TODO maybe pass the entity key in rather than getting it inside
    fn entity_description_is_ok(&self) -> (bool, HashableVec2) {
        if let Some(entity_key) = self.selected_entity {
            return (!self.entity_descriptions.contains(&self.entity_description) && self.entity_description.len() != 0, entity_key);
        }
        unreachable!(); // boy I hope so
    }

    fn do_entity_ok(&mut self, entity_key: HashableVec2) {
        println!("prev: {}, current: {}", self.prev_entity_description, self.entity_description);
        if self.prev_entity_description != self.entity_description {
            self.entity_descriptions.remove(&self.prev_entity_description);
        }
        self.show_entity_popup = false;
        self.entity_tiles.insert(entity_key, self.entity_description.clone());
        self.entity_descriptions.insert(self.entity_description.clone());
        self.entity_description = "".to_string();
    }

    fn handle_entity_popup(&mut self, ctx: &egui::Context) {
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
                            println!("prev: {}, current: {}", self.prev_entity_description, self.entity_description);
                            if (!description_is_ok && !self.prev_entity_description.len() == 0) || self.prev_entity_description.len() == 0 {
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_clear_confirmation(ctx);
        self.handle_entity_popup(ctx);
        self.handle_toplevel_input(ctx);
        self.top_panel(ctx);
        self.side_panel(ctx);
        self.plot_panel(ctx);
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
