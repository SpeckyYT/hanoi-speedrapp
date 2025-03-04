use eframe::egui::{vec2, Align, Align2, Area, Color32, Direction, FontId, Id, LayerId, Layout, Order, Painter, Pos2, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::{display::{PolesPosition, DISK_HEIGHT, POLE_COLOR, POLE_WIDTH, TEXT_COLOR, TEXT_OUTLINE_COLOR}, HanoiApp, PolesVec};

use super::DISK_WIDTH_MIN;

impl HanoiApp {
    pub fn draw_poles(&mut self, ui: &mut Ui, pointer_pos: Option<Pos2>) -> PolesVec<Response> {
        puffin::profile_function!();

        ui.scope(|ui| {
            let style = ui.style_mut();
            let previous_spacing = style.spacing.item_spacing;
            style.spacing.item_spacing = Vec2::new(0.0, 0.0);

            ui.columns(self.hanoi.poles_count, |uis| {
                uis.iter_mut()
                    .enumerate()
                    .map(|(i, ui)| {
                        ui.style_mut().spacing.item_spacing = previous_spacing;
                        let pole = self.draw_pole(ui, i).interact(Sense::click_and_drag());
                        if let Some(pointer_pos) = pointer_pos {
                            self.draw_pole_hover(ui, &pole, pointer_pos);
                        }
                        pole
                    })
                    .collect::<PolesVec<Response>>()
            })
        }).inner
    }

    pub fn draw_pole(&mut self, ui: &mut Ui, i: usize) -> Response {
        puffin::profile_function!();

        ui.with_layout(
            Layout::from_main_dir_and_cross_align(
                match self.poles_position {
                    PolesPosition::Bottom => Direction::BottomUp,
                    PolesPosition::Top => Direction::TopDown,
                },
                Align::Center,
            ),
            |ui| {
                puffin::profile_scope!("pole_layout");
                let max_width = ui.available_width();
                let max_height = ui.available_height();
                let spacing = ui.style_mut().spacing.item_spacing.y;
                let disk_height = DISK_HEIGHT.min((max_height - spacing * (self.hanoi.disks_count + 2) as f32) / (self.hanoi.disks_count as f32)).max(0.1);
                let mut disks_skipped = 0;

                self.hanoi.poles[i].iter().enumerate().for_each(|(j, &disk_number)| {
                    let is_drag = self.dragging_pole == Some(i);
                    let is_swift = self.swift_pole == Some(i);
                    let is_count = is_drag as usize + is_swift as usize;

                    if j >= self.hanoi.poles[i].len() - is_count {
                        disks_skipped += 1;
                    } else {
                        self.draw_disk(
                            ui,
                            disk_number,
                            max_width,
                            disk_height,
                        );
                    }
                });

                if self.show_poles {
                    let single_height = disk_height + spacing;
                    let pole_size = self.hanoi.poles[i].len();
                    let remaining_size = self.hanoi.disks_count + disks_skipped - pole_size + 1;
                    let remaining_height = remaining_size as f32 * single_height;
                    let size = vec2(POLE_WIDTH, remaining_height);
                    let (response, painter) = ui.allocate_painter(size, Sense::hover());
                    painter.rect_filled(response.rect, 0.0, POLE_COLOR);
                }

                ui.add_space(ui.available_height()); // this is useful drag and drop
            }
        ).response
    }

    pub fn draw_pole_hover(&mut self, ui: &mut Ui, pole: &Response, pointer_pos: Pos2) {
        if pole.rect.contains(pointer_pos) {
            Painter::new(ui.ctx().clone(), LayerId::background(), pole.rect)
                .rect(pole.rect, 20.0, Color32::from_black_alpha(16), Stroke::NONE, StrokeKind::Middle);
        }
    }

    pub fn calculate_disk_size(&self, disk_number: usize, max_width: f32, disk_height: f32) -> Vec2 {
        let width_step = (max_width - DISK_WIDTH_MIN) / self.hanoi.disks_count as f32;
        let width = DISK_WIDTH_MIN + disk_number as f32 * width_step;
        Vec2::new(
            width,
            disk_height,
        )
    }

    pub fn draw_disk(&self, ui: &mut Ui, disk_number: usize, max_width: f32, disk_height: f32) -> Response {
        puffin::profile_function!("draw_disk");

        let size = self.calculate_disk_size(disk_number, max_width, disk_height);
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let color = self.color_theme.to_color(disk_number, self.hanoi.disks_count);
        painter.rect_filled(response.rect, disk_height / 2.5, color);
        if self.disk_number {
            puffin::profile_scope!("disk_number");

            let center_pos = response.rect.center();
            let align = Align2::CENTER_CENTER;
            let disk_number = disk_number.to_string();
            let number_size = disk_height / 1.5;

            for x in -1..=1 {
                for y in -1..=1 {
                    if x == 0 || y == 0 { continue }
                    painter.text(
                        center_pos + vec2(x as f32, y as f32),
                        align,
                        &disk_number,
                        FontId::monospace(number_size),
                        TEXT_OUTLINE_COLOR,
                    );
                }
            }
            painter.text(
                center_pos,
                align,
                disk_number,
                FontId::monospace(number_size),
                TEXT_COLOR,
            );
        }
        response
    }

    pub fn draw_dragging_disk(&mut self, ui: &mut Ui) {
        if let Some(from) = self.dragging_pole {
            let position = ui.input(|i | i.pointer.interact_pos());

            if let (Some(&disk_number), Some(position)) = (self.hanoi.poles[from].last(), position) {
                let available_size = ui.ctx().available_rect();
                let disk_height = DISK_HEIGHT.min(available_size.height());
                let spacing_x = ui.style_mut().spacing.item_spacing.x;
                let max_width = available_size.width() / self.hanoi.poles_count as f32 - spacing_x * 2.0;
                let size = self.calculate_disk_size(disk_number, max_width, disk_height);

                Area::new(Id::new("dragging_disk"))
                    .order(Order::Foreground)
                    .interactable(false)
                    .fade_in(false)
                    .fixed_pos(position - size / 2.0)
                    .show(ui.ctx(), |ui| {
                        self.draw_disk(ui, disk_number, max_width, disk_height);
                    });
            }
        }
    }

    pub fn draw_swift_disk(&mut self, ui: &mut Ui) {
        // todo: this is turning into a copy and paste hell, multiple ways of playing should become modular
        if let Some(from) = self.swift_pole {
            if let Some(&disk_number) = self.hanoi.poles[from].last() {
                let available_size = ui.ctx().available_rect();
                let disk_height = DISK_HEIGHT.min(available_size.height());
                let spacing_x = ui.style_mut().spacing.item_spacing.x;
                let max_width = available_size.width() / self.hanoi.poles_count as f32 - spacing_x * 2.0;
                let size = self.calculate_disk_size(disk_number, max_width, disk_height);
                let position = Pos2::new(available_size.width() / 2.0, (size.y * 2.0).min(available_size.height() / 2.0));

                Area::new(Id::new("swift_disk"))
                    .order(Order::Foreground)
                    .interactable(false)
                    .fade_in(false)
                    .fixed_pos(position - size / 2.0)
                    .show(ui.ctx(), |ui| {
                        self.draw_disk(ui, disk_number, max_width, disk_height);
                    });
            }
        }
    }
}
