use std::{path::Path, time::Duration};

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
#[derive(Resource)]
pub struct DebugKeys(pub Vec<String>);
#[derive(Component)]
struct DebugMenu;
#[derive(Resource)]
pub struct DebugText {
    pub list: Vec<(String, String)>,
    pub color: Color,
}
impl Default for DebugText {
    fn default() -> Self {
        DebugText {
            list: vec![],
            color: Color::BLACK,
        }
    }
}
pub struct DebugTextPlugin;
#[derive(Resource)]
struct FPSTimer(Timer);

impl Plugin for DebugTextPlugin {
    fn build(&self, app: &mut App) {
        let mut fps_timer = Timer::from_seconds(0.05, TimerMode::Repeating);
        fps_timer.tick(Duration::from_secs_f32(2.));
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .insert_resource(FPSTimer(fps_timer))
            .add_systems(Startup, create_debug_overlay)
            .add_systems(Update, (update_debug_overlay, fps_update));
    }
}
pub fn change_debug_text(debug_text: &mut DebugText, key: &str, value: &str) {
    for line in &mut debug_text.list {
        if line.0 == key {
            line.1 = value.to_string();
        }
    }
}
fn construct_debug_string(debug_text: &DebugText) -> String {
    let mut string = String::new();
    for line in &debug_text.list {
        string.push_str(format!("{}: {}\n", line.0, line.1).as_str());
    }
    string
}
fn create_debug_overlay(
    mut commands: Commands,
    mut debug_text: ResMut<DebugText>,
    asset_server: Res<AssetServer>,
    debug_keys: Res<DebugKeys>,
) {
    let path = Path::new("./assets/fonts/Roboto/Roboto-Medium.ttf");
    if !path.exists() {
        eprintln!("Error: {:?} does not exist", path);
        return;
    }
    let font: Handle<Font> = asset_server.load("fonts/Roboto/Roboto-Medium.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 20.,
        color: debug_text.color,
    };
    debug_text.list.push(("FPS".to_string(), String::new()));
    for key in debug_keys.0.iter() {
        debug_text.list.push((key.to_string(), String::new()));
    }
    let string = construct_debug_string(debug_text.as_ref());
    commands
        .spawn((
            TextBundle::from_section(string, text_style)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(5.),
                    right: Val::Px(5.),
                    ..Default::default()
                })
                .with_text_justify(JustifyText::Right),
            DebugMenu,
        ))
        .insert(VisibilityBundle {
            visibility: Visibility::Hidden,
            ..Default::default()
        });
}
fn update_debug_overlay(
    mut query: Query<(&mut Text, &mut Visibility), With<DebugMenu>>,
    debug_text: Res<DebugText>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut text = query.single_mut();
    text.0.sections[0].value = construct_debug_string(debug_text.as_ref());
    if keyboard_input.just_pressed(KeyCode::F2) {
        match *text.1 {
            Visibility::Inherited => *text.1 = Visibility::Hidden,
            _ => *text.1 = Visibility::Inherited,
        }
    }
}
fn fps_update(
    mut timer: ResMut<FPSTimer>,
    time: Res<Time>,
    mut debug_text: ResMut<DebugText>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if timer.0.tick(time.delta()).finished() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps) = fps.smoothed() {
                change_debug_text(&mut debug_text, "FPS", &format!("{fps:.2}"));
            }
        };
    }
}
