// Adapted from Starling (https://github.com/starling-client/starling, `android` branch at
// 150a414b09a1, `crates/horizon-client-render/src/touch.rs`), written by Vasysik and the
// Starling contributors. Copyright (c) 2026 Ludovic Marie and the Horizon contributors, under the
// same MIT license as Horizon (LICENSE at the repository root).

use super::{Control, Mode, Placement, Role, TouchControls, layout};
use crate::gamepad::{Button, PadState};
use crate::hud::{HudAtlas, HudBatch, rgba};

impl TouchControls {
    pub fn draw(&self, b: &mut HudBatch, a: &HudAtlas) {
        let state = self.state();
        for control in Control::ALL {
            if !self.visible(control) {
                continue;
            }
            let p = self.placement(control);
            let color = rgba(255, 255, 255, alpha(p.opacity));
            if control == Control::Move {
                self.draw_move(b, a, p, &state, color);
                continue;
            }
            let bounds = square(b, p.center, p.size);
            let rect = p.rect(self.aspect());
            let pressed = self.held(control, &state)
                || self.fingers.values().any(|(_, point)| layout::contains(rect, *point));
            button(b, a, bounds, &self.label(control), pressed, color);
        }
        if let Some(point) = self.pointer() {
            let center = [point[0] * b.width, point[1] * b.height];
            let radius = self.settings.circle_size / 2.0;
            ring(b, a, center, radius + 0.7, rgba(0, 0, 0, 110));
            ring(b, a, center, radius, rgba(255, 255, 255, 210));
            let inner = mining_radius(radius, self.break_progress);
            if inner > 0.0 {
                disc(b, a, center, inner, rgba(255, 255, 255, 90));
                ring(b, a, center, inner, rgba(255, 255, 255, 220));
            }
        }
    }

    fn held(&self, control: Control, state: &PadState) -> bool {
        if control == Control::Sneak && self.sneaking { return true; }
        Self::button(control).is_some_and(|button| match button {
            Button::LeftTrigger => state.triggers[0] > 0.0,
            Button::RightTrigger => state.triggers[1] > 0.0,
            _ => state.held(button),
        })
    }

    // IKON MCPE BERBASIS SIMBOL (Karena kita tidak punya file gambar)
    fn label(&self, control: Control) -> String {
        let vertical = self.swimming || self.vertical_movement;
        match control {
            Control::Move => String::new(),
            Control::Pause => "II".to_owned(),
            Control::Inventory => "◼".to_owned(), // Kotak mirip tas
            Control::Debug => "F3".to_owned(),
            Control::Perspective => "F5".to_owned(),
            Control::Sprint => "»".to_owned(),
            Control::Jump if vertical => "▲".to_owned(),
            Control::Jump => "▲".to_owned(), // Panah atas
            Control::Sneak if vertical => "▼".to_owned(),
            Control::Sneak if self.mounted => "⤓".to_owned(),
            Control::Sneak => "▼".to_owned(), // Panah bawah
            Control::Attack => "⚔".to_owned(), // Pedang
            Control::Use => "◼".to_owned(), // Blok
            Control::Chat => "💬".to_owned(),
            Control::Settings => "⚙".to_owned(),
            Control::Offhand => "⇄".to_owned(),
            Control::Shield => "🛡".to_owned(),
        }
    }

    fn draw_move(&self, b: &mut HudBatch, a: &HudAtlas, p: Placement, state: &PadState, color: u32) {
        let anchor = self.fingers.values().find_map(|(role, _)| match role {
            Role::Move(anchor) => Some(*anchor),
            _ => None,
        });
        let center = match anchor {
            Some(anchor) if self.dynamic_joystick() => anchor,
            None if self.dynamic_joystick() => return,
            _ => p.center,
        };
        let [x, y, size, _] = square(b, center, p.size);
        if self.settings.mode == Mode::Dpad {
            let cell = size / 3.0;
            for ([dx, dy], label, pressed) in dpad(state.left) {
                let bounds = [x + cell * dx, y + cell * dy, cell, cell];
                button(b, a, bounds, label, pressed, color);
            }
            return;
        }
        let middle = [x + size / 2.0, y + size / 2.0];
        let [r, g, blue, opacity] = color.to_le_bytes();
        
        // Lingkaran Joystick MCPE
        disc(b, a, middle, size / 2.0, rgba(0, 0, 0, opacity / 4));
        ring(b, a, middle, size / 2.0, rgba(r, g, blue, opacity / 2));
        
        let knob_pos = knob(middle, state.left, size);
        disc(b, a, knob_pos, size * 0.18, rgba(r, g, blue, opacity / 3));
        ring(b, a, knob_pos, size * 0.18, color);
    }
}

fn dpad(left: [f32; 2]) -> [([f32; 2], &'static str, bool); 4] {
    [
        ([1.0, 0.0], "▲", left[1] > 0.2),
        ([0.0, 1.0], "◀", left[0] < -0.2),
        ([2.0, 1.0], "▶", left[0] > 0.2),
        ([1.0, 2.0], "▼", left[1] < -0.2),
    ]
}

fn knob(middle: [f32; 2], left: [f32; 2], size: f32) -> [f32; 2] {
    [middle[0] + left[0] * size / 4.0, middle[1] - left[1] * size / 4.0]
}

fn square(b: &HudBatch, center: [f32; 2], size: f32) -> [f32; 4] {
    let size = size * b.height;
    [center[0] * b.width - size / 2.0, center[1] * b.height - size / 2.0, size, size]
}

fn alpha(opacity: f32) -> u8 {
    (opacity.clamp(0.0, 1.0) * 255.0).round() as u8
}

// Tombol Bulat Transparan MCPE
fn button(b: &mut HudBatch, _a: &HudAtlas, bounds: [f32; 4], label: &str, pressed: bool, color: u32) {
    let [x, y, w, h] = bounds;
    let [r, g, blue, opacity] = color.to_le_bytes();
    let center = [x + w / 2.0, y + h / 2.0];
    let radius = w / 2.0;

    let fill_alpha = if pressed { opacity / 2 } else { opacity / 4 };
    let border_alpha = if pressed { 255 } else { opacity };
    
    ring(b, _a, center, radius + 1.0, rgba(0, 0, 0, 150));
    disc(b, _a, center, radius, rgba(0, 0, 0, fill_alpha));
    ring(b, _a, center, radius, rgba(r, g, blue, border_alpha));
    
    if label.is_empty() { return; }
    
    let text = if pressed { rgba(r / 2 + 127, g / 2 + 127, blue / 2, 255) } else { color };
    let drop = if pressed { 1.0 } else { 0.0 };
    
    // Perbesar sedikit ukuran teks karena sekarang isinya simbol, bukan kata
    let width = HudBatch::text_width(_a, label);
    let start = b.vertices.len();
    
    b.text(_a, label, center[0] - width / 2.0, center[1] - 5.0 + drop, text, true);
    
    let fit_width = (w - 4.0).max(1.0) / width.max(1.0);
    let height_cap = ((h - 4.0).max(1.0) / 8.0).min(1.0);
    let scale = fit_width.min(height_cap) * 1.5; // Skala 1.5x biar ikonnya gede
    b.scale_since(start, center[0], center[1] + drop, scale, scale);
}

fn mining_radius(radius: f32, progress: f32) -> f32 {
    if progress.is_finite() { (radius - 1.5).max(0.0) * progress.clamp(0.0, 1.0) } else { 0.0 }
}

fn circle_points() -> &'static [[f32; 2]; 65] {
    static POINTS: std::sync::OnceLock<[[f32; 2]; 65]> = std::sync::OnceLock::new();
    POINTS.get_or_init(|| {
        std::array::from_fn(|i| {
            let angle = i as f32 / 64.0 * std::f32::consts::TAU;
            [angle.cos(), angle.sin()]
        })
    })
}

fn ring(b: &mut HudBatch, a: &HudAtlas, center: [f32; 2], radius: f32, color: u32) {
    let at = |p: [f32; 2], r: f32| [center[0] + p[0] * r, center[1] + p[1] * r];
    let (outer, inner) = (radius + 0.7, (radius - 0.7).max(0.0));
    for s in circle_points().windows(2) {
        b.fill_quad(a, [at(s[0], outer), at(s[1], outer), at(s[1], inner), at(s[0], inner)], color);
    }
}

fn disc(b: &mut HudBatch, a: &HudAtlas, center: [f32; 2], radius: f32, color: u32) {
    let at = |p: [f32; 2]| [center[0] + p[0] * radius, center[1] + p[1] * radius];
    for s in circle_points().windows(2) {
        b.fill_quad(a, [center, at(s[0]), at(s[1]), center], color);
    }
}
