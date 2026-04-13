use eframe::egui;

const COLOR_1: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(0, 0, 0, 255);
const COLOR_2: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(0, 255, 0, 64);
const COLOR_3: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(0, 255, 0, 128);
const COLOR_4: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(0, 255, 0, 255);
const COLOR_5: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(255, 0, 0, 255);

const CORNER_RADIUS: f32 = 0.0;
const LINE_WIDTH: f32 = 1.0;

const FONT_SIZE_1: f32 = 4.0;
const FONT_SIZE_2: f32 = 8.0;
const FONT_SIZE_3: f32 = 12.0;
const FONT_SIZE_4: f32 = 16.0;

pub struct Input1 {
    text_state: TextInput,
}

impl Input1 {
    pub const fn new() -> Self {
        Self {
            text_state: TextInput::new(),
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        text: &mut String,
        placeholder: &str,
        is_disabled: bool,
    ) {
        let (rect, res) = ui.allocate_exact_size(egui::vec2(100.0, 50.0), egui::Sense::all());

        let hovered = res.hovered();
        let clicked = res.clicked();
        let focused = res.has_focus();

        if clicked {
            res.request_focus();
        }

        if focused {
            self.text_state.handle_input_events(ui, text);
        }

        let color = if is_disabled {
            COLOR_2
        } else if clicked {
            COLOR_3
        } else {
            COLOR_4
        };

        ui.painter().rect_stroke(
            rect,
            CORNER_RADIUS,
            egui::Stroke::new(LINE_WIDTH, color),
            egui::StrokeKind::Inside,
        );

        let font_id = egui::FontId::monospace(FONT_SIZE_4);
        if text.is_empty() {
            ui.painter().text(
                rect.left_center(),
                egui::Align2::LEFT_CENTER,
                placeholder,
                font_id,
                COLOR_3,
            );
        } else {
            ui.painter().text(
                rect.left_center(),
                egui::Align2::LEFT_CENTER,
                text.clone(),
                font_id.clone(),
                COLOR_4,
            );

            Self::draw_cursor(ui, rect, text, font_id, self.text_state.cursor_pos);
        }

        drow_hover_and_focus(ui, rect, focused, hovered);
    }

    fn draw_cursor(
        ui: &mut egui::Ui,
        text_rect: egui::Rect,
        text: &String,
        font_id: egui::FontId,
        cursor_pos: usize,
    ) {
        // Create a galley to measure the text height
        let galley = ui.painter().layout_no_wrap(
            text[..cursor_pos].to_string(),
            font_id,
            egui::Color32::WHITE,
        );

        let pos = galley.size().x;
        let cursor_x = text_rect.left() + pos;

        let text_height = galley.size().y;
        let text_center_y = text_rect.center().y;

        let cursor_top = egui::Pos2::new(cursor_x, text_center_y - text_height);
        let cursor_bottom = egui::Pos2::new(cursor_x, text_center_y + text_height);

        ui.painter().line_segment(
            [cursor_top, cursor_bottom],
            egui::Stroke::new(LINE_WIDTH, COLOR_4),
        );
    }
}

pub fn input_password(ui: &mut egui::Ui, text: &mut String, placeholder: &str) {
    ui.horizontal(|ui| {
        // ui.text_edit_singleline(text);
        // make it global const
        let hide = egui::Image::new(egui::include_image!(
            "../../accounting_ui/assets/icons/hide.png"
        ));
        let show = egui::Image::new(egui::include_image!(
            "../../accounting_ui/assets/icons/show.png"
        ));

        // ui.add(egui::Button::image(hide));
    });
}

pub fn error_label(ui: &mut egui::Ui, text: &str) {
    ui.label(text);
}

pub struct Btn1;

impl Btn1 {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, text: &str, is_disabled: bool) -> bool {
        let (rect, res) = ui.allocate_exact_size(egui::vec2(100.0, 50.0), egui::Sense::all());

        let hovered = res.hovered();
        let clicked = res.clicked();
        let focused = res.has_focus();

        let color = if is_disabled {
            COLOR_2
        } else if clicked {
            COLOR_3
        } else {
            COLOR_4
        };
        ui.painter().rect_filled(rect, CORNER_RADIUS, color);

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(FONT_SIZE_4),
            COLOR_1,
        );

        drow_hover_and_focus(ui, rect, focused, hovered);

        clicked
    }
}

pub struct Btn2;

impl Btn2 {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, text: &str, is_disabled: bool) -> bool {
        let (rect, res) = ui.allocate_exact_size(egui::vec2(100.0, 50.0), egui::Sense::all());

        let hovered = res.hovered();
        let clicked = res.clicked();
        let focused = res.has_focus();

        let color = if is_disabled {
            COLOR_2
        } else if clicked {
            COLOR_3
        } else {
            COLOR_4
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(FONT_SIZE_4),
            color,
        );
        ui.painter().rect_stroke(
            rect,
            CORNER_RADIUS,
            egui::Stroke::new(LINE_WIDTH, color),
            egui::StrokeKind::Inside,
        );

        drow_hover_and_focus(ui, rect, focused, hovered);

        clicked
    }
}

fn drow_hover_and_focus(ui: &mut egui::Ui, rect: egui::Rect, focused: bool, hovered: bool) {
    let make_rect = |color: egui::Color32| {
        ui.painter().rect_stroke(
            rect.expand(3.0),
            CORNER_RADIUS,
            egui::Stroke::new(LINE_WIDTH, color),
            egui::StrokeKind::Inside,
        );
    };
    if focused {
        make_rect(COLOR_4);
    } else if hovered {
        make_rect(COLOR_3);
    }
}

///////////////////////////////////////////
// Helper struct for actions
#[derive(Default)]
struct InputActions {
    text_input: Option<String>,
    backspace: bool,
    delete: bool,
    arrow_left: bool,
    arrow_right: bool,
    home: bool,
    end: bool,
    enter: bool,
    tab: bool,
    escape: bool,
    copy: bool,
    cut: bool,
    paste: bool,
    select_all: bool,
}

pub struct TextInput {
    cursor_pos: usize,
    selection_start: Option<usize>,
}

// Helper methods
impl TextInput {
    const fn new() -> Self {
        Self {
            cursor_pos: 0,
            selection_start: None,
        }
    }

    fn handle_input_events(&mut self, ui: &mut egui::Ui, text: &mut String) {
        let ctx = ui.ctx().clone();
        let mut actions = InputActions::default();

        // STEP 1: Collect all input state FIRST (read-only)
        let modifiers = ui.input(|i| i.modifiers);

        // Get keyboard events
        ui.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Text(chars) => {
                        actions.text_input = Some(chars.clone());
                    }

                    egui::Event::Copy => actions.copy = true,
                    egui::Event::Cut => actions.cut = true,
                    egui::Event::Paste(chars) => {
                        actions.paste = true;
                        actions.text_input = Some(chars.clone());
                    }

                    egui::Event::Key { key, pressed, .. } => {
                        if *pressed {
                            match key {
                                egui::Key::Backspace => actions.backspace = true,
                                egui::Key::Delete => actions.delete = true,
                                egui::Key::ArrowLeft => actions.arrow_left = true,
                                egui::Key::ArrowRight => actions.arrow_right = true,
                                egui::Key::Home => actions.home = true,
                                egui::Key::End => actions.end = true,
                                egui::Key::Enter => actions.enter = true,
                                egui::Key::Tab => actions.tab = true,
                                egui::Key::Escape => actions.escape = true,
                                egui::Key::A if modifiers.ctrl => actions.select_all = true,
                                _ => {}
                            }
                        }
                    }

                    _ => {}
                }
            }
        });

        // STEP 2: Process clipboard operations (requires external access)
        if actions.copy && self.has_selection() {
            if let Some(selected) = self.get_selected_text(text) {
                ctx.copy_text(selected);
            }
            self.selection_start = None;
        }

        if actions.cut && self.has_selection() {
            if let Some(selected) = self.get_selected_text(text) {
                ctx.copy_text(selected);
                self.delete_selection(text);
            }
        }

        // need to fix , i dont think
        // if actions.paste {
        //     if let Some(contents) = ctx.input(|i| i.raw.pasted_text()) {
        //         if self.has_selection() {
        //             self.delete_selection(text);
        //         }
        //         text.insert_str(self.cursor_pos, &contents);
        //         self.cursor_pos += contents.len();
        //     }
        // }

        if actions.select_all {
            self.selection_start = Some(0);
            self.cursor_pos = text.len();
        }

        // STEP 3: Process text input
        if let Some(chars) = actions.text_input {
            if self.has_selection() {
                self.delete_selection(text);
            }
            text.insert_str(self.cursor_pos, &chars);
            self.cursor_pos += chars.len();
        }

        // need to fix
        // STEP 4: Process navigation keys
        if actions.arrow_left {
            if modifiers.shift && !self.has_selection() {
                self.selection_start = Some(self.cursor_pos);
            } else if !modifiers.shift {
                self.selection_start = None;
            }

            if modifiers.ctrl {
                // Jump by word (simplified)
                self.cursor_pos = self.prev_word_boundary(text, self.cursor_pos);
            } else if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
            }
        }

        // need to fix
        if actions.arrow_right {
            if modifiers.shift && !self.has_selection() {
                self.selection_start = Some(self.cursor_pos);
            } else if !modifiers.shift {
                self.selection_start = None;
            }

            if modifiers.ctrl {
                // Jump by word (simplified)
                self.cursor_pos = self.next_word_boundary(text, self.cursor_pos);
            } else if self.cursor_pos < text.len() {
                self.cursor_pos += 1;
            }
        }

        if actions.home {
            if modifiers.shift && !self.has_selection() {
                self.selection_start = Some(self.cursor_pos);
            } else if !modifiers.shift {
                self.selection_start = None;
            }
            self.cursor_pos = 0;
        }

        if actions.end {
            if modifiers.shift && !self.has_selection() {
                self.selection_start = Some(self.cursor_pos);
            } else if !modifiers.shift {
                self.selection_start = None;
            }
            self.cursor_pos = text.len();
        }

        // STEP 5: Process deletion
        if actions.backspace {
            if self.has_selection() {
                self.delete_selection(text);
            } else if self.cursor_pos > 0 {
                if modifiers.ctrl {
                    // Delete word (simplified)
                    let start = self.prev_word_boundary(text, self.cursor_pos);
                    text.replace_range(start..self.cursor_pos, "");
                    self.cursor_pos = start;
                } else {
                    text.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            }
        }

        if actions.delete {
            if self.has_selection() {
                self.delete_selection(text);
            } else if self.cursor_pos < text.len() {
                if modifiers.ctrl {
                    // Delete word (simplified)
                    let end = self.next_word_boundary(text, self.cursor_pos);
                    text.replace_range(self.cursor_pos..end, "");
                } else {
                    text.remove(self.cursor_pos);
                }
            }
        }

        // STEP 6: Handle Enter/Escape/Tab (return to caller)
        if actions.enter {
            // You can handle this in the return value
        }

        if actions.escape {
            self.selection_start = None;
            // Could also surrender focus
        }
    }

    fn has_selection(&self) -> bool {
        self.selection_start.is_some() && self.selection_start.unwrap() != self.cursor_pos
    }

    fn get_selected_text(&self, text: &str) -> Option<String> {
        if let Some(start) = self.selection_start {
            let (s, e) = (start.min(self.cursor_pos), start.max(self.cursor_pos));
            if s != e {
                return Some(text[s..e].to_string());
            }
        }
        None
    }

    fn delete_selection(&mut self, text: &mut String) {
        if let Some(start) = self.selection_start {
            let (s, e) = (start.min(self.cursor_pos), start.max(self.cursor_pos));
            if s != e {
                text.replace_range(s..e, "");
                self.cursor_pos = s;
                self.selection_start = None;
            }
        }
    }

    fn prev_word_boundary(&self, text: &str, pos: usize) -> usize {
        // Simplified word boundary detection
        let mut i = pos.saturating_sub(1);
        while i > 0 && !text[i..].chars().next().unwrap().is_whitespace() {
            i -= 1;
        }
        i
    }

    fn next_word_boundary(&self, text: &str, pos: usize) -> usize {
        // Simplified word boundary detection
        let mut i = pos;
        while i < text.len() && !text[i..].chars().next().unwrap().is_whitespace() {
            i += 1;
        }
        i
    }
}
