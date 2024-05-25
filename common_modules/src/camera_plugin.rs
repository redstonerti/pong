use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
#[derive(Resource)]
pub struct CameraSettings {
    speed: f32,
    scale_speed: f32,
    scale_multiplier: f32,
    max_zoom: i32,
    min_zoom: i32,
    target_scale: f32,
    zoom_smoothing: f32,
    position_difference: Vec2,
    middle_mouse_start_position: Vec2,
    target_position: Vec2,
    desired_scale: f32,
    is_going_to_target_position: bool,
}
impl Default for CameraSettings {
    fn default() -> Self {
        CameraSettings {
            speed: 200.,
            scale_speed: 1.2,
            max_zoom: 40,
            min_zoom: 30,
            scale_multiplier: 2.,
            target_scale: 5.,
            zoom_smoothing: 0.075,
            position_difference: Vec2::new(0., 0.),
            middle_mouse_start_position: Vec2::new(0., 0.),
            target_position: Vec2::new(0., 0.),
            desired_scale: 1.,
            is_going_to_target_position: false,
        }
    }
}
#[derive(Component)]
pub struct MainCamera;
#[derive(Resource)]
pub struct CursorWorldPosition(pub Vec2);
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings::default())
            .insert_resource(CursorWorldPosition {
                0: Vec2::new(0., 0.),
            })
            .add_systems(Startup, camera_setup)
            .add_systems(Update, camera_movement)
            .add_systems(Update, definitely_my_cursor_system_which_isnt_stolen);
    }
}
fn camera_setup(
    mut commands: Commands,
    mut camera_settings: ResMut<CameraSettings>,
    camera_query: Query<Entity, With<Camera2d>>,
) {
    if camera_query.is_empty() {
        commands.spawn((
            Camera2dBundle {
                projection: OrthographicProjection {
                    scale: camera_settings.scale_multiplier,
                    ..Default::default()
                },
                ..Default::default()
            },
            MainCamera,
        ));
    }
    camera_settings.target_scale = camera_settings.scale_multiplier;
}
fn camera_movement(
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut scroll_event_reader: EventReader<MouseWheel>,
    mut camera_settings: ResMut<CameraSettings>,
    cursor_world_position: Res<CursorWorldPosition>,
) {
    let mut scroll_direction = 0;
    for event in scroll_event_reader.read() {
        scroll_direction = match event.unit {
            _ => {
                if event.y >= 1. {
                    1
                } else if event.y == 0. {
                    0
                } else {
                    -1
                }
            }
        }
    }
    //Move Camera
    let mut camera = query.single_mut();
    if camera_settings.is_going_to_target_position {
        if (camera.1.scale - camera_settings.desired_scale).abs() <= 0.1
            && ((camera.0.translation.x - camera_settings.target_position.x).powi(2)
                + (camera.0.translation.y - camera_settings.target_position.y).powi(2))
            .sqrt()
                <= 10.
        {
            camera_settings.is_going_to_target_position = false;
        } else {
            if camera.1.scale > camera_settings.desired_scale {
                camera.1.scale -= (camera.1.scale - camera_settings.desired_scale)
                    / camera_settings.zoom_smoothing
                    / 2.
                    * time.delta_seconds();
            } else {
                camera.1.scale -= (camera.1.scale - camera_settings.desired_scale)
                    / camera_settings.zoom_smoothing
                    / 2.
                    * time.delta_seconds();
            }
            camera.0.translation.x -= (camera.0.translation.x - camera_settings.target_position.x)
                * 10.
                * time.delta_seconds();
            camera.0.translation.y -= (camera.0.translation.y - camera_settings.target_position.y)
                * 10.
                * time.delta_seconds();
            return ();
        }
    }
    let world_screen_displacement: Vec2 = Vec2 {
        x: cursor_world_position.0.x - camera.0.translation.x,
        y: cursor_world_position.0.y - camera.0.translation.y,
    };
    if scroll_direction != 0 {
        let mut scale_difference: f32 = camera_settings.target_scale;
        camera_settings.target_scale *= camera_settings.scale_speed.powi(-scroll_direction);
        camera_settings.target_scale = camera_settings
            .target_scale
            .max(
                1. / camera_settings.scale_speed.powi(camera_settings.max_zoom)
                    * camera_settings.scale_multiplier,
            )
            .min(
                camera_settings.scale_speed.powi(camera_settings.min_zoom)
                    * camera_settings.scale_multiplier,
            );
        scale_difference = camera_settings.target_scale / scale_difference;
        //Calculate world screen displacement from mouse
        let x_displacement: f32 =
            world_screen_displacement.x - world_screen_displacement.x * scale_difference;
        let y_displacement: f32 =
            world_screen_displacement.y - world_screen_displacement.y * scale_difference;
        camera_settings.position_difference.x += x_displacement;
        camera_settings.position_difference.y += y_displacement;
    }
    //Move camera to mouse
    let frame_difference_x = camera_settings.position_difference.x * time.delta_seconds()
        / camera_settings.zoom_smoothing;
    let frame_difference_y = camera_settings.position_difference.y * time.delta_seconds()
        / camera_settings.zoom_smoothing;
    camera.0.translation.x += frame_difference_x;
    camera.0.translation.y += frame_difference_y;
    camera_settings.position_difference.x -= frame_difference_x;
    camera_settings.position_difference.y -= frame_difference_y;
    camera.1.scale -= ((camera.1.scale - camera_settings.target_scale)
        / camera_settings.zoom_smoothing)
        * time.delta_seconds();

    //Middle mouse drag
    if mouse_input.just_pressed(MouseButton::Middle) {
        camera_settings.middle_mouse_start_position = cursor_world_position.0;
    }
    if mouse_input.pressed(MouseButton::Middle) {
        camera.0.translation.x +=
            (camera_settings.middle_mouse_start_position.x - cursor_world_position.0.x) / 2.;
        camera.0.translation.y +=
            (camera_settings.middle_mouse_start_position.y - cursor_world_position.0.y) / 2.;
    }
    if mouse_input.pressed(MouseButton::Middle) {
        return ();
    }
    let mut x_axis: f32 = 0.;
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        x_axis = -1.;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        x_axis = 1.;
    }
    let mut y_axis: f32 = 0.;
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        y_axis = 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        y_axis = -1.;
    }
    let mut speed_boost: f32 = 1.0;
    if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
        speed_boost = 5.;
    }
    camera.0.translation.x +=
        x_axis * time.delta_seconds() * camera_settings.speed * speed_boost * camera.1.scale;
    camera.0.translation.y +=
        y_axis * time.delta_seconds() * camera_settings.speed * speed_boost * camera.1.scale;
}
fn definitely_my_cursor_system_which_isnt_stolen(
    // need to get window dimensions
    windows: Query<&Window>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_position: ResMut<CursorWorldPosition>,
) {
    let wnd = windows.single();

    let (camera, camera_transform) = q_camera.get_single().unwrap();

    let mouse_pos_2d = wnd
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
        .unwrap_or_else(|| mouse_position.0);
    mouse_position.0 = mouse_pos_2d;
}
fn _calculate_world_coordinates(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
    mouse_position: &mut CursorWorldPosition,
) {
    if let Some(mut screen_pos) = window.cursor_position() {
        screen_pos.y *= -1.;
        // get the size of the window
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();
        mouse_position.0 = world_pos;
    }
}
//fn target_position_listener(target_call_reader: ResMut) {}
pub fn go_to_target_position(
    target: Vec2,
    camera_settings: &mut CameraSettings,
    desired_scale: Option<f32>,
) {
    *camera_settings = CameraSettings::default();
    camera_settings.is_going_to_target_position = true;
    camera_settings.target_position = target;
    match desired_scale {
        Some(value) => {
            camera_settings.desired_scale = value * camera_settings.scale_multiplier;
            camera_settings.target_scale = camera_settings.desired_scale;
        }
        None => {
            camera_settings.desired_scale =
                camera_settings.target_scale * camera_settings.scale_multiplier;
        }
    }
}
