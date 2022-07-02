use eframe::{egui, epi};
use egui::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use chrono::{Utc};

const DATA: &str = "data.json";

fn save_data_to_file(tasks: &Tasks) {
    let mut output = File::create(DATA).unwrap();
    let serialized = serde_json::to_string_pretty(tasks).unwrap();
    write!(output, "{}", serialized).unwrap();
}

fn read_data_from_file(value: File) -> Tasks {
    let buffered = BufReader::new(value);

    let mut string = "".to_string();

    for line in buffered.lines() {
        string += &line.unwrap();
    }

    return serde_json::from_str(&string).unwrap();
}

pub fn drag_source(ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui)) {
    let is_being_dragged = ui.memory().is_being_dragged(id);

    if !is_being_dragged {
        let response = ui.scope(body).response;

        // Check for drags:
        let response = ui.interact(response.rect, id, Sense::drag());
        if response.hovered() {
            ui.output().cursor_icon = CursorIcon::Grab;
        }
    } else {
        ui.output().cursor_icon = CursorIcon::Grabbing;

        // Paint the body to a new layer:
        let layer_id = LayerId::new(Order::Tooltip, id);
        let response = ui.with_layer_id(layer_id, body).response;

        // Now we move the visuals of the body to where the mouse is.
        // Normally you need to decide a location for a widget first,
        // because otherwise that widget cannot interact with the mouse.
        // However, a dragged component cannot be interacted with anyway
        // (anything with `Order::Tooltip` always gets an empty `Response`)
        // So this is fine!

        if let Some(pointer_pos) = ui.input().pointer.interact_pos() {
            let delta = pointer_pos - response.rect.center();
            ui.ctx().translate_layer(layer_id, delta);
        }
    }
}

pub fn drop_target<R>(
    ui: &mut Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let is_being_dragged = ui.memory().is_anything_being_dragged();

    let margin = Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(Shape::Noop);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

    let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    let mut fill = style.bg_fill;
    let mut stroke = style.bg_stroke;
    if is_being_dragged && !can_accept_what_is_being_dragged {
        // gray out:
        fill = color::tint_color_towards(fill, ui.visuals().window_fill());
        stroke.color = color::tint_color_towards(stroke.color, ui.visuals().window_fill());
    }

    ui.painter().set(
        where_to_put_background,
        epaint::RectShape {
            corner_radius: style.corner_radius,
            fill,
            stroke,
            rect,
        },
    );

    InnerResponse::new(ret, response)
}

#[derive(PartialEq, Eq)]
enum Menu {
    First,
    Second,
    Third,
}

impl Default for Menu {
    fn default() -> Self {
        Self::First
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Task {
    task: String,
    date: String,
    time: String,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            task: "".to_string(),
            date: Utc::today().format("%d.%m.%y").to_string(),
            time: "0h:0m".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
struct Tasks {
    columns: Vec<Vec<Task>>,
}

impl Default for Tasks {
    fn default() -> Self {
        Self {
            columns: vec![
                vec![
                    Task {
                        task: "Education Rust".to_string(),
                        date: Utc::today().format("%d.%m.%y").to_string(),
                        time: "0h:0m".to_string(),
                    },
                    Task {
                        task: "Education C++".to_string(),
                        date: Utc::today().format("%d.%m.%y").to_string(),
                        time: "0h:0m".to_string(),
                    },
                    Task {
                        task: "Education Assembler".to_string(),
                        date: Utc::today().format("%d.%m.%y").to_string(),
                        time: "0h:0m".to_string(),
                    },
                    Task {
                        task: "Education English".to_string(),
                        date: Utc::today().format("%d.%m.%y").to_string(),
                        time: "0h:0m".to_string(),
                    },
                    Task {
                        task: "Education Painting".to_string(),
                        date: Utc::today().format("%d.%m.%y").to_string(),
                        time: "0h:0m".to_string(),
                    },
                ],
                vec![],
                vec![],
            ],
        }
    }
}

#[derive(Default)]
struct SelectedTask {
    column: usize,
    row: usize,
    element: Task,
}

#[derive(Default)]
struct MyApp {
    menu_elements: Menu,
    tasks: Tasks,
    selected_task: SelectedTask,
    input_task: Task,
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        match File::open(DATA) {
            Ok(value) => {
                self.tasks = read_data_from_file(value);
            }
            Err(e) => {
                let tasks = Tasks::default();
                save_data_to_file(&tasks);
            }
        }

        let mut fonts = egui::FontDefinitions::default();

        fonts
            .family_and_size
            .insert(TextStyle::Heading, (FontFamily::Proportional, 32.0));

        fonts
            .family_and_size
            .insert(TextStyle::Button, (FontFamily::Proportional, 20.0));

        fonts
            .family_and_size
            .insert(TextStyle::Body, (FontFamily::Proportional, 20.0));

        ctx.set_fonts(fonts);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.menu_elements, Menu::First, "Список задач");
                ui.selectable_value(&mut self.menu_elements, Menu::Second, "Создать задачу");
            });
            ui.add_space(10.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.menu_elements {
            Menu::First => {
                ui.heading("Статусы задач");
                ui.add_space(10.0);
                ui.separator();

                let column_titles = ["planned", "in progress", "done"];

                let id_source = "my_drag_and_drop_app";
                let mut source_col_row = None;
                let mut drop_col = None;
                ui.columns(self.tasks.columns.len(), |col_ui| {
                    for (col_idx, column) in self.tasks.columns.clone().into_iter().enumerate() {
                        let ui = &mut col_ui[col_idx];

                        ui.heading(column_titles[col_idx]);
                        ui.add_space(10.0);

                        let can_accept_what_is_being_dragged = true; // We accept anything being dragged (for now) ¯\_(ツ)_/¯
                        let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                            ui.set_min_size(vec2(64.0, 100.0));
                            for (row_idx, item) in column.iter().enumerate() {
                                let item_id = Id::new(id_source).with(col_idx).with(row_idx);

                                ui.horizontal(|ui| {
                                    drag_source(ui, item_id, |ui| {
                                        let response = ui.add(
                                            Label::new(format!(
                                                "{} | {} | {}",
                                                item.task, item.date, item.time
                                            ))
                                            .sense(Sense::click()),
                                        );
                                        response.context_menu(|ui| {
                                            if ui.button("Remove").clicked() {
                                                self.tasks.columns[col_idx].remove(row_idx);
                                                ui.close_menu();
                                            }
                                        });
                                    });

                                    if ui.button("more").clicked() {
                                        self.selected_task = SelectedTask {
                                            column: col_idx,
                                            row: row_idx,
                                            element: self.tasks.columns[col_idx][row_idx].clone(),
                                        }
                                    }

                                    if &self.selected_task.element.task
                                        == &self.tasks.columns[col_idx][row_idx].task
                                    {
                                        if ui.button("edit").clicked() {
                                            self.menu_elements = Menu::Third;
                                        }

                                        if ui.button("delete").clicked() {
                                            self.tasks.columns[col_idx].remove(row_idx);
                                        }
                                    }
                                });

                                ui.separator();

                                if ui.memory().is_being_dragged(item_id) {
                                    source_col_row = Some((col_idx, row_idx));
                                }
                            }
                        })
                        .response;

                        let is_being_dragged = ui.memory().is_anything_being_dragged();
                        if is_being_dragged
                            && can_accept_what_is_being_dragged
                            && response.hovered()
                        {
                            drop_col = Some(col_idx);
                        }
                    }
                });

                if let Some((source_col, source_row)) = source_col_row {
                    if let Some(drop_col) = drop_col {
                        if ui.input().pointer.any_released() {
                            // do the drop:
                            let item = self.tasks.columns[source_col].remove(source_row);
                            self.tasks.columns[drop_col].push(item);
                            save_data_to_file(&self.tasks);
                        }
                    }
                }
            }
            Menu::Second => {
                ui.heading("Создать новую задачу");
                ui.add_space(10.0);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("задача:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.input_task.task).desired_width(120.0),
                    );
                    ui.label("дата:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.input_task.date).desired_width(120.0),
                    );
                    ui.label("времени потрачено:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.input_task.time).desired_width(120.0),
                    );

                    let popup_id = ui.make_persistent_id("some_id_for_create_btn");
                    let response = ui.button("создать");
                    if response.clicked() {
                        if self.input_task.task.len() > 0 {
                            ui.memory().toggle_popup(popup_id);
                            self.tasks.columns[0].push(self.input_task.clone());
                            self.input_task.task = "".to_string();
                        }

                        save_data_to_file(&self.tasks);
                    }

                    egui::popup::popup_below_widget(ui, popup_id, &response, |ui| {
                        ui.set_min_width(200.0); // if you want to control the size
                        ui.label("task was created");
                    });
                });
            }
            Menu::Third => {
                ui.heading("Редактировать");
                ui.add_space(10.0);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("задача:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.selected_task.element.task)
                            .desired_width(120.0),
                    );
                    ui.label("дата:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.selected_task.element.date).desired_width(120.0),
                    );
                    ui.label("времени потрачено:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.selected_task.element.time)
                            .desired_width(120.0),
                    );

                    let popup_id = ui.make_persistent_id("some_id_for_create_btn");
                    let response = ui.button("редактировать");
                    if response.clicked() {
                        self.tasks.columns[self.selected_task.column]
                            .remove(self.selected_task.row);
                        if self.selected_task.element.task.len() > 0 {
                            ui.memory().toggle_popup(popup_id);
                            self.tasks.columns[self.selected_task.column].push(self.selected_task.element.clone());
                            self.selected_task = SelectedTask::default();
                        }

                        save_data_to_file(&self.tasks);
                    }

                    egui::popup::popup_below_widget(ui, popup_id, &response, |ui| {
                        ui.set_min_width(200.0); // if you want to control the size
                        ui.label("task was edited");
                    });
                });
            }
        });
    }
}

fn main() {
    let app = MyApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
