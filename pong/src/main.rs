use bevy::app::AppExit;
use bevy::app::{App, Startup, Update};
use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use bevy_prototype_lyon::prelude::*;
use common_modules::debug_text_plugin::{change_debug_text, DebugKeys, DebugText};
use rand::Rng;
use std::time::Duration;

const IS_FULLSCREEN: bool = false;
const PLAY_AREA: Vec2 = vec2(800., 436.);

#[derive(Resource)]
struct LoadedSounds(HashMap<String, Handle<AudioSource>>);

#[derive(Debug)]
enum ControlMethod {
    Keyboard,
    Mouse,
}

#[derive(Resource, Debug)]
struct MouseCoords(Vec2);

#[derive(Resource)]
struct BallTimer {
    timer: Timer,
    should_tick: bool,
}
#[derive(Component)]
struct Ball {
    velocity: Vec2,
}
#[derive(Component, Debug)]
pub struct Score {
    player: Player,
}
#[derive(Debug, PartialEq)]
enum Player {
    Left,
    Right,
}
enum Orientation {
    Landscape,
    Portrait,
}
#[derive(Resource)]
pub struct Settings {
    fullscreen: bool,
    max_paddle_speed: f32,
    paddle_acceleration: f32,
    friction: f32,
    paddle_size: Vec2,
    paddle_x: f32,
    orientation: Orientation,
    ball_size: f32,
    max_spawn_speed: Vec2,
    min_spawn_speed: Vec2,
    score_spacing: f32,
    speed_multiplier: f32,
    mouse_control_area: Vec2,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: IS_FULLSCREEN,
            max_paddle_speed: 500.,
            paddle_acceleration: 3000.,
            friction: 700.,
            paddle_x: 350.,
            paddle_size: vec2(10., 50.),
            orientation: Orientation::Landscape,
            ball_size: 10.,
            max_spawn_speed: vec2(300., 300.),
            min_spawn_speed: vec2(200., 0.),
            score_spacing: 20.,
            speed_multiplier: 1.1,
            mouse_control_area: vec2(30., 20.),
        }
    }
}
#[derive(Resource)]
struct DigitSpriteSheet(Handle<TextureAtlasLayout>);
#[derive(Resource, Debug)]
struct GameData {
    left_dir: f32,
    right_dir: f32,
    left_y: f32,
    right_y: f32,
    left_score: u32,
    right_score: u32,
    should_update_scores: bool,
    player_controlled_by_mouse: Option<Player>,
}
#[derive(Component, Debug)]
struct Paddle {
    speed: f32,
    player: Player,
}
impl Default for GameData {
    fn default() -> Self {
        Self {
            left_dir: 0.,
            right_dir: 0.,
            left_y: 0.,
            right_y: 0.,
            left_score: 0,
            right_score: 0,
            should_update_scores: false,
            player_controlled_by_mouse: None,
        }
    }
}
impl FromWorld for DigitSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let texture_atlas = TextureAtlasLayout::from_grid(
            vec2(3., 5.),
            5,
            2,
            Some(vec2(1., 1.)),
            Some(vec2(1., 1.)),
        );
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        Self(texture_atlas_handle)
    }
}
pub struct PongPlugin;
impl Plugin for PongPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            .init_resource::<DigitSpriteSheet>()
            .insert_resource(LoadedSounds { 0: HashMap::new() })
            .insert_resource(DebugText {
                color: Color::WHITE,
                ..Default::default()
            })
            .insert_resource(MouseCoords(Vec2::ZERO))
            .insert_resource(BallTimer {
                timer: Timer::new(Duration::from_millis(1000), TimerMode::Repeating),
                should_tick: false,
            })
            .insert_resource(DebugKeys(vec![
                "Camera Scale".into(),
                "Window Dimensions".into(),
                "Play Area".into(),
                "Mouse Coords".into(),
                "Directions".into(),
            ]))
            .insert_resource(Settings::default())
            .insert_resource(GameData::default())
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(Startup, (setup, spawn_background, load_sounds))
            .add_systems(
                Update,
                (
                    handle_actions,
                    accelerate_paddles,
                    scale_game,
                    update_ball,
                    spawn_ball,
                    update_scores,
                    get_cursor_coords,
                ),
            );
    }
}
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        mode: if IS_FULLSCREEN {
                            bevy::window::WindowMode::BorderlessFullscreen
                        } else {
                            bevy::window::WindowMode::Windowed
                        },
                        title: "Pong".into(),
                        resolution: PLAY_AREA.into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            common_modules::debug_text_plugin::DebugTextPlugin,
            PongPlugin,
        ))
        .run();
}
fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    sprite_atlas: Res<DigitSpriteSheet>,
    settings: Res<Settings>,
    mut ball_timer: ResMut<BallTimer>,
) {
    let camera = Camera::default();
    commands.spawn(Camera2dBundle {
        camera,
        ..Default::default()
    });
    spawn_score(
        Player::Left,
        &mut commands,
        asset_server.as_ref(),
        sprite_atlas.as_ref(),
        0,
        &settings,
        0,
    );
    spawn_score(
        Player::Right,
        &mut commands,
        asset_server.as_ref(),
        sprite_atlas.as_ref(),
        0,
        &settings,
        0,
    );
    spawn_paddle(Player::Left, &mut commands, settings.as_ref());
    spawn_paddle(Player::Right, &mut commands, settings.as_ref());
    ball_timer.should_tick = true;
}
fn load_sounds(asset_server: Res<AssetServer>, mut loaded_sounds: ResMut<LoadedSounds>) {
    for i in 0..6 {
        loaded_sounds.0.insert(
            format!("{i}"),
            asset_server.load(&format!("sounds/{i}.mp3")),
        );
    }
    for i in 0..3 {
        loaded_sounds.0.insert(
            format!("hit{i}"),
            asset_server.load(&format!("sounds/hit{i}.mp3")),
        );
    }
    loaded_sounds.0.insert(
        format!("death"),
        asset_server.load(&format!("sounds/death.mp3")),
    );
}
fn spawn_ball(
    mut commands: Commands,
    settings: Res<Settings>,
    mut ball_timer: ResMut<BallTimer>,
    time: Res<Time>,
) {
    if ball_timer.should_tick {
        ball_timer.timer.tick(time.delta());
    } else {
        return;
    }
    if !ball_timer.timer.just_finished() {
        return;
    } else {
        ball_timer.should_tick = false;
    }
    let mut thread_rng = rand::thread_rng();
    let square = spawn_square(
        vec2(settings.ball_size, settings.ball_size),
        thread_rng.gen_range(
            (-PLAY_AREA.y / 2. + settings.ball_size * 2.)
                ..(PLAY_AREA.y / 2. - settings.ball_size * 2.),
        ),
        &mut commands,
    );
    let direction = match thread_rng.gen_range(0..2) {
        0 => -1,
        _ => 1,
    } as f32;
    commands.get_entity(square).unwrap().insert(Ball {
        velocity: vec2(
            thread_rng.gen_range(settings.min_spawn_speed.x..settings.max_spawn_speed.x)
                * direction,
            thread_rng.gen_range(settings.min_spawn_speed.y..settings.max_spawn_speed.y)
                * direction,
        ),
    });
}
fn spawn_background(mut commands: Commands) {
    let line_amount = 30;
    let section_height = PLAY_AREA.y / line_amount as f32;
    let line_height = section_height * 5. / 7.;
    let line_width = line_height / 7.;
    let mut current_position = PLAY_AREA.y / 2. - line_height / 2.;
    for _ in 0..line_amount {
        spawn_square(
            vec2(line_width, line_height),
            current_position,
            &mut commands,
        );
        current_position -= section_height;
    }
    let mut thread_rng = rand::thread_rng();
    let backgroudn_color = match thread_rng.gen_range(0..3) {
        0 => Color::rgb(0., 0.4, 0.4),
        1 => Color::rgb(0.2, 0., 0.8),
        _ => Color::rgb(1., 0.2, 0.6),
    };
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shapes::Rectangle {
                extents: PLAY_AREA,
                origin: RectangleOrigin::Center,
            }),
            spatial: SpatialBundle {
                transform: Transform::from_translation(vec3(0., 0., -2.)),
                ..Default::default()
            },
            ..Default::default()
        },
        Fill::color(backgroudn_color),
    ));
}
fn spawn_square(size: Vec2, position: f32, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Rectangle {
                    extents: size,
                    origin: RectangleOrigin::Center,
                }),
                spatial: SpatialBundle {
                    transform: Transform::from_translation(vec3(0., position, 0.)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Fill::color(Color::WHITE),
        ))
        .id()
}

fn num_length(num: u32) -> u32 {
    num.checked_ilog10().unwrap_or(0) + 1
}
fn get_digits(n: usize) -> Vec<usize> {
    fn x_inner(n: usize, xs: &mut Vec<usize>) {
        if n >= 10 {
            x_inner(n / 10, xs);
        }
        xs.push(n % 10);
    }
    let mut xs = Vec::new();
    x_inner(n, &mut xs);
    xs
}

fn update_ball(
    mut ball: Query<(&mut Ball, &mut Transform, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    mut ball_timer: ResMut<BallTimer>,
    mut game_data: ResMut<GameData>,
    mut loaded_sounds: ResMut<LoadedSounds>,
) {
    let mut ball = match ball.get_single_mut() {
        Ok(ball) => ball,
        Err(_) => return,
    };
    let mut ball_pos = vec2(ball.1.translation.x, ball.1.translation.y);
    ball_pos.x += ball.0.velocity.x * time.delta_seconds();
    ball_pos.y += ball.0.velocity.y * time.delta_seconds();

    let paddle_x: f32;
    let paddle_y: f32;
    if ball_pos.x < 0. {
        paddle_x = -settings.paddle_x;
        paddle_y = game_data.left_y;
    } else {
        paddle_x = settings.paddle_x;
        paddle_y = game_data.right_y;
    }
    let mut thread_rng = rand::thread_rng();
    if ball_pos.x > paddle_x - settings.paddle_size.x
        && ball_pos.x < paddle_x + settings.paddle_size.x
        && ball_pos.y > paddle_y - settings.paddle_size.y
        && ball_pos.y < paddle_y + settings.paddle_size.y
    {
        let speed = (ball.0.velocity.x.powi(2) + ball.0.velocity.y.powi(2)).sqrt()
            * settings.speed_multiplier;
        let pos_difference = vec2(
            ball.1.translation.x - paddle_x,
            ball.1.translation.y - paddle_y,
        );
        let angle = libm::atan2(pos_difference.y as f64, pos_difference.x as f64);
        ball.0.velocity.x = speed * angle.cos() as f32;
        ball.0.velocity.y = speed * angle.sin() as f32;
        let sound = loaded_sounds
            .0
            .get(&format!("hit{}", thread_rng.gen_range(0..3)))
            .unwrap();
        commands.spawn(AudioBundle {
            source: sound.clone(),
            ..default()
        });
        ball_pos.x += ball.0.velocity.x * time.delta_seconds();
        ball_pos.y += ball.0.velocity.y * time.delta_seconds();
    }

    if ball_pos.x - settings.ball_size > PLAY_AREA.x / 2.
        || ball_pos.x + settings.ball_size < -PLAY_AREA.x / 2.
    {
        commands.get_entity(ball.2).unwrap().despawn_recursive();
        ball_timer.should_tick = true;
        let sound = loaded_sounds.0.get(&format!("death")).unwrap();
        commands.spawn(AudioBundle {
            source: sound.clone(),
            ..default()
        });
        if ball_pos.x < 0. {
            game_data.right_score += 1;
        } else {
            game_data.left_score += 1;
        }
        game_data.should_update_scores = true;
        return;
    }
    if ball_pos.y + settings.ball_size / 2. > PLAY_AREA.y / 2.
        || ball_pos.y - settings.ball_size / 2. < -PLAY_AREA.y / 2.
    {
        ball.0.velocity.y = -ball.0.velocity.y * (settings.speed_multiplier).sqrt().sqrt();
        ball.0.velocity.x *= (settings.speed_multiplier).sqrt().sqrt();
        ball_pos.x += ball.0.velocity.x * time.delta_seconds();
        ball_pos.y += ball.0.velocity.y * time.delta_seconds();
        let sound = loaded_sounds
            .0
            .get(&format!("{}", thread_rng.gen_range(0..6)))
            .unwrap();
        commands.spawn(AudioBundle {
            source: sound.clone(),
            ..default()
        });
    }

    ball.1.translation.x = ball_pos.x;
    ball.1.translation.y = ball_pos.y;
}
fn spawn_paddle(player: Player, commands: &mut Commands, settings: &Settings) {
    commands.spawn((
        (
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Rectangle {
                    extents: settings.paddle_size,
                    origin: RectangleOrigin::Center,
                }),
                spatial: SpatialBundle {
                    transform: match player {
                        Player::Left => {
                            Transform::from_translation(vec3(-settings.paddle_x, 0., 0.))
                        }
                        Player::Right => {
                            Transform::from_translation(vec3(settings.paddle_x, 0., 0.))
                        }
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            Fill::color(Color::WHITE),
        ),
        Paddle { speed: 0., player },
    ));
}
fn update_scores(
    mut game_data: ResMut<GameData>,
    mut scores: Query<(&mut Score, &mut TextureAtlas, &mut Transform)>,
    mut commands: Commands,
    mut sprite_atlas: ResMut<DigitSpriteSheet>,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
) {
    if !game_data.should_update_scores {
        return;
    } else {
        game_data.should_update_scores = false;
    }
    let mut left_scores = vec![];
    let mut right_scores = vec![];
    for score in scores.iter_mut() {
        match score.0.player {
            Player::Left => left_scores.push(score),
            Player::Right => right_scores.push(score),
        }
    }

    let left_score_len = num_length(game_data.left_score);
    let left_digits = get_digits(game_data.left_score as usize);
    if left_score_len > left_scores.len() as u32 {
        spawn_score(
            Player::Left,
            &mut commands,
            &asset_server,
            &mut sprite_atlas,
            left_score_len - 1,
            &settings,
            left_digits[left_digits.len() - 1],
        );
        for i in 0..(left_scores.len()) {
            left_scores[i].2.translation.x -= settings.score_spacing;
        }
    }
    for i in 0..(left_scores.len()) {
        left_scores[i].1.index = left_digits[i];
    }
    let right_score_len = num_length(game_data.right_score);
    let right_digits = get_digits(game_data.right_score as usize);
    if right_score_len > right_scores.len() as u32 {
        spawn_score(
            Player::Right,
            &mut commands,
            &asset_server,
            &mut sprite_atlas,
            right_score_len - 1,
            &settings,
            right_digits[right_digits.len() - 1],
        );
        for i in 0..(right_scores.len()) {
            right_scores[i].2.translation.x -= settings.score_spacing;
        }
    }
    for i in 0..(right_scores.len()) {
        right_scores[i].1.index = right_digits[i];
    }
}
fn spawn_score(
    player: Player,
    commands: &mut Commands,
    asset_server: &AssetServer,
    sprite_atlas: &DigitSpriteSheet,
    position: u32,
    settings: &Settings,
    index: usize,
) -> Entity {
    let sprite = asset_server.load("spritesheets/digits.png");
    commands
        .spawn((
            SpriteSheetBundle {
                atlas: TextureAtlas {
                    layout: sprite_atlas.0.clone(),
                    index,
                },
                texture: sprite,
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(vec2(27., 45.)),
                    ..Default::default()
                },
                transform: match &player {
                    Player::Left => Transform::from_translation(vec3(
                        -200. + position as f32 * settings.score_spacing,
                        150.,
                        -1.,
                    )),
                    Player::Right => Transform::from_translation(vec3(
                        200. + position as f32 * settings.score_spacing,
                        150.,
                        -1.,
                    )),
                },
                ..Default::default()
            },
            Score { player },
        ))
        .id()
}
fn accelerate_paddles(
    mut query: Query<(&mut Transform, &mut Paddle)>,
    settings: Res<Settings>,
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
    mouse_pos: Res<MouseCoords>,
) {
    for mut paddle in query.iter_mut() {
        let direction = match paddle.1.player {
            Player::Left => game_data.left_dir,
            Player::Right => game_data.right_dir,
        };
        paddle.1.speed = (paddle.1.speed
            + direction as f32 * settings.paddle_acceleration * time.delta_seconds())
        .min(settings.max_paddle_speed)
        .max(-settings.max_paddle_speed);
        let mut friction = settings.friction;
        if let Some(player) = &game_data.player_controlled_by_mouse {
            if player == &paddle.1.player {
                friction /= direction;
            }
        }
        if paddle.1.speed > 0. {
            paddle.1.speed = (paddle.1.speed - friction * time.delta_seconds()).max(0.);
        }
        if paddle.1.speed < 0. {
            paddle.1.speed = (paddle.1.speed + friction * time.delta_seconds()).min(0.);
        }
        if let Some(player) = &game_data.player_controlled_by_mouse {
            if player == &paddle.1.player {
                if paddle.0.translation.y > mouse_pos.0.y {
                    paddle.0.translation.y = (paddle.0.translation.y
                        + paddle.1.speed * time.delta_seconds())
                    .max(mouse_pos.0.y);
                } else if paddle.0.translation.y < mouse_pos.0.y {
                    paddle.0.translation.y = (paddle.0.translation.y
                        + paddle.1.speed * time.delta_seconds())
                    .min(mouse_pos.0.y);
                }
                if paddle.0.translation.y == mouse_pos.0.y {
                    paddle.1.speed = 0.;
                }
            } else {
                paddle.0.translation.y += paddle.1.speed * time.delta_seconds();
            }
        } else {
            paddle.0.translation.y += paddle.1.speed * time.delta_seconds();
        }

        let max_y: f32 = (PLAY_AREA.y - settings.paddle_size.y) / 2.;
        if paddle.0.translation.y > max_y {
            paddle.1.speed = 0.;
            paddle.0.translation.y = max_y;
        } else if paddle.0.translation.y < -max_y {
            paddle.1.speed = 0.;
            paddle.0.translation.y = -max_y;
        }
        match paddle.1.player {
            Player::Left => game_data.left_y = paddle.0.translation.y,
            Player::Right => game_data.right_y = paddle.0.translation.y,
        };
    }
}
fn scale_game(
    mut camera: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    window: Query<&Window>,
    mut debug_text: ResMut<DebugText>,
    mut settings: ResMut<Settings>,
    mut scores: Query<(&mut Transform, &mut TextureAtlas), Without<OrthographicProjection>>,
) {
    let window = window.single();
    let mut camera = camera.single_mut();
    let ratio1: f32;
    let ratio2: f32;
    let aspect_ratio = window.width() / window.height();
    let scale: f32;
    let mut update_scores = false;
    if aspect_ratio < 1. {
        ratio1 = window.width() / PLAY_AREA.y;
        ratio2 = window.height() / PLAY_AREA.x;
        camera.0.rotation = Quat::from_euler(EulerRot::XYZ, 0., 0., 0.5 * 3.145);
        if ratio1 < ratio2 {
            scale = 1. / ratio1;
        } else {
            scale = 1. / ratio2;
        }
        if let Orientation::Landscape = settings.orientation {
            settings.orientation = Orientation::Portrait;
            update_scores = true;
        }
    } else {
        ratio1 = window.width() / PLAY_AREA.x;
        ratio2 = window.height() / PLAY_AREA.y;
        camera.0.rotation = Quat::default();
        if ratio1 < ratio2 {
            scale = 1. / ratio1;
        } else {
            scale = 1. / ratio2;
        }
        if let Orientation::Portrait = settings.orientation {
            settings.orientation = Orientation::Landscape;
            update_scores = true;
        }
    }
    camera.1.scale = scale;
    if update_scores {
        for mut score in scores.iter_mut() {
            match settings.orientation {
                Orientation::Landscape => score.0.rotation = Quat::default(),
                Orientation::Portrait => {
                    score.0.rotation = Quat::from_euler(EulerRot::XYZ, 0., 0., 0.5 * 3.145)
                }
            }
        }
    }
    change_debug_text(&mut debug_text, "Ratio1", &format!("{:.2}", ratio1));
    change_debug_text(&mut debug_text, "Ratio2", &format!("{:.2}", ratio2));
    change_debug_text(
        &mut debug_text,
        "Camera Scale",
        &format!("{}", camera.1.scale),
    );
    change_debug_text(
        &mut debug_text,
        "Window Dimensions",
        &format!("({}, {})", window.width(), window.height()),
    );
    change_debug_text(&mut debug_text, "Play Area", &PLAY_AREA.to_string());
}
fn handle_actions(
    mut exit: EventWriter<AppExit>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut settings: ResMut<Settings>,
    window: Query<&mut Window>,
    mut game_data: ResMut<GameData>,
    mouse_pos: Res<MouseCoords>,
    mut debug_text: ResMut<DebugText>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
    if keyboard_input.just_pressed(KeyCode::KeyF) || keyboard_input.just_pressed(KeyCode::F11) {
        settings.fullscreen = !settings.fullscreen;
        fullscreen(window, settings.fullscreen);
    }
    game_data.left_dir = 0.;
    game_data.right_dir = 0.;
    if keyboard_input.pressed(match settings.orientation {
        Orientation::Landscape => KeyCode::KeyW,
        Orientation::Portrait => KeyCode::KeyD,
    }) {
        game_data.left_dir += 1.;
    }
    if keyboard_input.pressed(match settings.orientation {
        Orientation::Landscape => KeyCode::KeyS,
        Orientation::Portrait => KeyCode::KeyA,
    }) {
        game_data.left_dir -= 1.;
    }
    if keyboard_input.pressed(match settings.orientation {
        Orientation::Landscape => KeyCode::ArrowUp,
        Orientation::Portrait => KeyCode::ArrowRight,
    }) {
        game_data.right_dir += 1.;
    }
    if keyboard_input.pressed(match settings.orientation {
        Orientation::Landscape => KeyCode::ArrowDown,
        Orientation::Portrait => KeyCode::ArrowLeft,
    }) {
        game_data.right_dir -= 1.;
    }
    if mouse_input.pressed(MouseButton::Left) {
        if mouse_pos.0.x < 0. {
            game_data.left_dir = (mouse_pos.0.y - game_data.left_y)
                .max(-settings.mouse_control_area.y)
                .min(settings.mouse_control_area.y)
                / settings.mouse_control_area.y;
            game_data.player_controlled_by_mouse = Some(Player::Left);
        } else {
            game_data.right_dir = (mouse_pos.0.y - game_data.right_y)
                .max(-settings.mouse_control_area.y)
                .min(settings.mouse_control_area.y)
                / settings.mouse_control_area.y;
            game_data.player_controlled_by_mouse = Some(Player::Right);
        }
        change_debug_text(
            &mut debug_text,
            "Directions",
            &format!("({:.2}, {:.2})", game_data.left_dir, game_data.right_dir),
        );
    } else {
        game_data.player_controlled_by_mouse = None;
    }
}
fn fullscreen(mut window: Query<&mut Window>, fullscreen: bool) {
    match window.get_single_mut() {
        Ok(mut window) => {
            if fullscreen {
                window.mode = bevy::window::WindowMode::BorderlessFullscreen
            } else {
                window.mode = bevy::window::WindowMode::Windowed
            }
        }
        Err(err) => eprintln!("Failed to get window: {err:#?}"),
    };
}
fn get_cursor_coords(
    mut mycoords: ResMut<MouseCoords>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut debug_text: ResMut<DebugText>,
) {
    let (camera, camera_transform) = camera.single();
    let window = window.single();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
        change_debug_text(
            &mut debug_text,
            "Mouse Coords",
            &format!("{world_position:.2?}"),
        );
    }
}
