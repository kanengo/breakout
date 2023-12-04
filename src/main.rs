use bevy::{
    prelude::*, sprite::MaterialMesh2dBundle,
};


const PADDLE_SIZE: Vec3 = Vec3::new(80.0, 10.0, 0.0);
const PADDLE_COLOR: Color = Color::WHITE;
const PADDLE_SPEED:f32 = 400.0;

const BRICK_SIZE: Vec2 = Vec2::new(10.0, 10.0);
const BRICK_COLOR: Color = Color::GREEN;

const BACKGROUND_COLOR: Color = Color::BLUE;

const RIGHT_EDGE: f32 = 500.0;
const LEFT_EDGE: f32 = -500.0;
const TOP_EDGE: f32 = 300.0;
const BOTTOM_EDGE: f32 = -300.0;

const BALL_COLOR: Color = Color::WHITE;
const BALL_SIZE: Vec3 = Vec3::new(10.0, 10.0, 0.0);

#[derive(Resource)]
struct BrickCounter(u16);

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Brick;

#[derive(Component)]
struct Chunk;

#[derive(Resource)]
struct ShowWindowInfoTimer(Timer);

impl ShowWindowInfoTimer {
    fn new() -> Self {
        Self(Timer::from_seconds(3.0,TimerMode::Repeating))
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(BrickCounter(100))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(ShowWindowInfoTimer::new())
        .add_systems(Startup, setup)
        .add_systems(Update, show_info)
        .add_systems(Update,(move_paddle).chain())
        .run();
}

fn show_info(windows: Query<&Window>, time: Res<Time>, mut timer: ResMut<ShowWindowInfoTimer>) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    for window in windows.iter() {
        let focused = window.focused;
        println!("Window is focused: {:?}", focused);

        // The size after scaling:
        let logical_width = window.width();
        let logical_height = window.height();
        println!("Logical size: {:?} x {:?}", logical_width, logical_height);

        // The size before scaling:
        let physical_width = window.physical_width();
        let physical_height = window.physical_height();
        println!("physical size: {:?} x {:?}", physical_width, physical_height);

        // Cursor position in logical sizes, this would return None if our
        // cursor is outside of the window:
        if let Some(logical_cursor_position) = window.cursor_position() {
            println!("Logical cursor position: {:?}", logical_cursor_position);
        }

        // Cursor position in physical sizes, this would return None if our
        // cursor is outside of the window:
        if let Some(physical_cursor_position) = window.physical_cursor_position() {
            println!("Physical cursor position: {:?}", physical_cursor_position);
        }
    }
}

fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {

    //camera
    commands.spawn(Camera2dBundle::default());

    //paddle
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -240.0,0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
        Velocity(Vec2::new(0.0,0.0)),
    ));

    //ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(Vec3::new(0.0,0.0,0.0)).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
    ));

    //chunks

}

fn move_paddle(
    keyboard_inpit: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Paddle>>,
    time: Res<Time>
) {
    let (mut paddle_transform, mut paddle_velocity) = query.single_mut();

    if keyboard_inpit.pressed(KeyCode::Left) {
        paddle_velocity.0 = Vec2::new(-1.0, 0.0);
    } else if keyboard_inpit.pressed(KeyCode::Right) {
        paddle_velocity.0 = Vec2::new(1.0, 0.0);
    } else {
        paddle_velocity.0 = Vec2::new(0.0, 0.0);
    }

    paddle_velocity.0.x *= PADDLE_SPEED * time.delta_seconds();
    

    let new_paddle_position_x = 
        paddle_transform.translation.x + paddle_velocity.0.x;

    let left_bound = LEFT_EDGE - paddle_transform.scale.x / 2.0;
    let right_bound = RIGHT_EDGE - paddle_transform.scale.x / 2.0;

    paddle_transform.translation.x = new_paddle_position_x.clamp(left_bound, right_bound);
}