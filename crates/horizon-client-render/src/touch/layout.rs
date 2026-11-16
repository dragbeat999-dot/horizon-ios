// Adapted from Starling (https://github.com/starling-client/starling, `android` branch at
// 150a414b09a1, `crates/horizon-client-render/src/touch/layout.rs`), written by Vasysik and the
// Starling contributors. Copyright (c) 2026 Ludovic Marie and the Horizon contributors, under the
// same MIT license as Horizon (LICENSE at the repository root).

//! The touch layout: which on-screen controls exist, where each one sits for
//! each control scheme, and the player's saved choices.
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    #[default]
    Crosshair,
    Tap,
    Dpad,
}

impl Mode {
    pub const ALL: [Self; 3] = [Self::Crosshair, Self::Tap, Self::Dpad];
    pub const SCHEMES: [(Self, bool, &'static str); 4] = [
        (Self::Crosshair, false, "Joystick & aim crosshair"),
        (Self::Tap, false, "Joystick & tap to interact"),
        (Self::Dpad, false, "D-Pad & tap to interact"),
        (Self::Dpad, true, "D-Pad & aim crosshair"),
    ];
    pub fn index(self) -> usize { self as usize }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Move, Sprint, Jump, Sneak, Attack, Use, Chat, Pause, Settings, Inventory, Debug, Perspective, Offhand, Shield,
}

pub const CONTROLS: usize = Control::ALL.len();

impl Control {
    pub const ALL: [Self; 14] = [
        Self::Move, Self::Sprint, Self::Jump, Self::Sneak, Self::Attack, Self::Use, 
        Self::Chat, Self::Pause, Self::Settings, Self::Inventory, Self::Debug, Self::Perspective, Self::Offhand, Self::Shield,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Move => "Movement",
            Self::Sprint => "Sprint",
            Self::Jump => "Jump / fly up",
            Self::Sneak => "Sneak / fly down",
            Self::Attack => "Attack / mine",
            Self::Use => "Use / place",
            Self::Chat => "Chat",
            Self::Pause => "Pause",
            Self::Settings => "Touch settings",
            Self::Inventory => "Inventory",
            Self::Debug => "Debug overlay (F3)",
            Self::Perspective => "Camera perspective (F5)",
            Self::Offhand => "Swap hands",
            Self::Shield => "Raise shield",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub center: [f32; 2],
    pub size: f32,
    pub opacity: f32,
}

impl Placement {
    pub const fn new(x: f32, y: f32, size: f32) -> Self {
        Self { center: [x, y], size, opacity: 0.65 }
    }

    pub fn rect(self, aspect: f32) -> [f32; 4] {
        let w = self.size / aspect.max(0.1);
        [self.center[0] - w / 2.0, self.center[1] - self.size / 2.0, w, self.size]
    }

    pub fn sanitize(&mut self) {
        self.size = finite_or(self.size, 0.15).clamp(0.045, 0.45);
        self.opacity = finite_or(self.opacity, 0.65).clamp(0.0, 1.0);
        for c in &mut self.center {
            *c = finite_or(*c, 0.5).clamp(0.025, 0.975);
        }
    }
}

pub const FOLLOW_SCHEME: u8 = 0;
pub const SHOWN: u8 = 1;
pub const HIDDEN: u8 = 2;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub split_controls: bool,
    pub left_handed: bool,
    pub swap_jump_sneak: bool,
    pub sprint_button: bool,
    pub sprint_with_joystick: bool,
    pub offhand_button: bool,
    pub shield_button: bool,
    pub button_visibility: [u8; CONTROLS],
    pub layouts: [[Placement; CONTROLS]; 3],
    pub sensitivity: f32,
    pub sensitivity_horizontal: f32,
    pub sensitivity_vertical: f32,
    pub sprint_toggle: bool,
    pub crosshair_gestures: bool,
    pub attack_button_look: bool,
    pub contextual_interact: bool,
    pub dynamic_joystick: bool,
    pub dynamic_area_x: f32,
    pub dynamic_area_y: f32,
    pub button_scale: f32,
    pub circle_size: f32,
}

impl Default for Settings {
    fn default() -> Self {
        // Posisi 100% MCPE (Wajik / Diamond)
        let row = [
            Placement::new(0.15, 0.65, 0.35),   // Move (Joystick lebih besar)
            Placement::new(0.90, 0.65, 0.15),   // Sprint (Kanan atas)
            Placement::new(0.90, 0.85, 0.15),   // Jump (Kanan bawah)
            Placement::new(0.75, 0.85, 0.15),   // Sneak (Kiri bawah)
            Placement::new(0.75, 0.55, 0.15),   // Attack (Kiri atas)
            Placement::new(0.85, 0.75, 0.15),   // Use (Tengah-tengah)
            Placement::new(0.46, 0.055, 0.075), // Chat
            Placement::new(0.50, 0.055, 0.075), // Pause
            Placement::new(0.54, 0.055, 0.075), // Settings
            Placement::new(0.73, 0.95, 0.075),  // Inventory
            Placement::new(0.42, 0.055, 0.075), // Debug
            Placement::new(0.58, 0.055, 0.075), // Perspective
            Placement::new(0.80, 0.96, 0.075),  // Offhand
            Placement::new(0.60, 0.85, 0.12),   // Shield
        ];
        let mut layouts = [row; 3];
        layouts[Mode::Dpad.index()][Control::Sneak as usize] = Placement::new(0.15, 0.75, 0.10);
        layouts[Mode::Dpad.index()][Control::Jump as usize] = Placement::new(0.90, 0.85, 0.15);
        Self {
            mode: Mode::Crosshair,
            split_controls: false,
            left_handed: false,
            swap_jump_sneak: false,
            sprint_button: true,
            sprint_with_joystick: false,
            offhand_button: false,
            shield_button: true,
            button_visibility: [FOLLOW_SCHEME; CONTROLS],
            layouts,
            sensitivity: 1.0,
            sensitivity_horizontal: 1.0,
            sensitivity_vertical: 1.0,
            sprint_toggle: false,
            crosshair_gestures: false,
            attack_button_look: true,
            contextual_interact: false,
            dynamic_joystick: false,
            dynamic_area_x: 0.25,
            dynamic_area_y: 1.0,
            button_scale: 1.0,
            circle_size: 32.0,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        let mut settings: Self = std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        settings.sanitize();
        settings
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(temp, path)
    }

    pub fn sanitize(&mut self) {
        if self.mode != Mode::Dpad { self.split_controls = false; }
        for v in &mut self.button_visibility { if *v > HIDDEN { *v = FOLLOW_SCHEME; } }
        for p in self.layouts.iter_mut().flatten() { p.sanitize(); }
        for v in [&mut self.sensitivity, &mut self.sensitivity_horizontal, &mut self.sensitivity_vertical] {
            *v = finite_or(*v, 1.0).clamp(0.1, 3.0);
        }
        self.button_scale = finite_or(self.button_scale, 1.0).clamp(0.5, 2.0);
        self.circle_size = finite_or(self.circle_size, 32.0).clamp(16.0, 64.0);
        self.dynamic_area_x = finite_or(self.dynamic_area_x, 0.25).clamp(0.0, 1.0);
        self.dynamic_area_y = finite_or(self.dynamic_area_y, 1.0).clamp(0.0, 1.0);
    }

    pub fn crosshair(&self) -> bool {
        self.mode == Mode::Crosshair || self.split_controls
    }

    pub fn visible(&self, c: Control) -> bool {
        match self.button_visibility[c as usize] {
            SHOWN => return true,
            HIDDEN => return false,
            _ => {}
        }
        match c {
            Control::Offhand => self.offhand_button,
            Control::Shield => self.shield_button && !self.visible(Control::Use),
            Control::Attack | Control::Use => self.crosshair(),
            Control::Sprint => self.sprint_button && self.mode != Mode::Dpad && !self.sprint_with_joystick,
            _ => true,
        }
    }

    pub fn slot(&self, c: Control) -> usize {
        let swap = self.mode == Mode::Dpad && self.swap_jump_sneak;
        match c {
            Control::Jump if swap => Control::Sneak as usize,
            Control::Sneak if swap => Control::Jump as usize,
            _ => c as usize,
        }
    }

    pub fn placement(&self, c: Control) -> Placement {
        let mut p = self.layouts[self.mode.index()][self.slot(c)];
        if self.left_handed { p.center[0] = 1.0 - p.center[0]; }
        p
    }
}

pub fn contains([x, y, w, h]: [f32; 4], p: [f32; 2]) -> bool {
    (x..=x + w).contains(&p[0]) && (y..=y + h).contains(&p[1])
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}
