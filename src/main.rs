use bevy::{
    prelude::*, sprite::{MaterialMesh2dBundle, collide_aabb::collide, Anchor},
};


const PADDLE_SIZE: Vec3 = Vec3::new(80.0, 10.0, 0.0);
const PADDLE_COLOR: Color = Color::WHITE;
const PADDLE_SPEED:f32 = 400.0;

const BRICK_SIZE: Vec3 = Vec3::new(10.0, 10.0,0.0);
const BRICK_COLOR: Color = Color::GREEN;
const GAP_BETWEEN_BRICKS: f32 = 2.0;

const BACKGROUND_COLOR: Color = Color::BLACK;

const RIGHT_EDGE: f32 = 640.0;
const LEFT_EDGE: f32 = -640.0;
const TOP_EDGE: f32 = 360.0;
const BOTTOM_EDGE: f32 = -360.0;

const BALL_COLOR: Color = Color::WHITE;
const BALL_SIZE: Vec3 = Vec3::new(10.0, 10.0, 0.0);
const BALL_SPEED: f32 = 200.0;

const WALL_COLOR: Color = Color::GRAY;

const CHUNK_SIZE: Vec3 = Vec3::new(120.0,120.0,0.0);

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

#[derive(Component,Default)]
struct Chunk;

#[derive(Component)]
struct WallBlock;

#[derive(Resource)]
struct ShowWindowInfoTimer(Timer);

impl ShowWindowInfoTimer {
    fn new() -> Self {
        Self(Timer::from_seconds(3.0,TimerMode::Repeating))
    }
}

enum CollilderType {
    WALL,
    CHUNK,
    BRICK,
    PADDLE,
}

#[derive(Component, Deref, DerefMut)]
struct Collilder(CollilderType);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(BrickCounter(100))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(ShowWindowInfoTimer::new())
        .add_systems(Startup, setup)
        .add_systems(Update, (show_info, gizmos_system))
        .add_systems(FixedUpdate,(
            (move_paddle, apply_velocity,)
                .chain().before(check_paddle_position_edge),
            check_paddle_position_edge,
            check_ball_position_edge,
            check_collider_paddle,
        ))
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {

    //camera
    commands.spawn(Camera2dBundle::default());
    
    

    //paddle
    let paddle_translation = Vec3::new(0.0, -240.0, 0.0);
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: paddle_translation,
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
        Collilder(CollilderType::PADDLE),
    ));

    // let mut rng = rand::thread_rng();

    //ball
    let ball_translation = Vec3::new(paddle_translation.x, paddle_translation.y, 0.0);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(ball_translation).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
        Velocity(Vec2::new(BALL_SPEED, BALL_SPEED)),
    ));

    //chunks
    let chunk = commands.spawn((
        SpatialBundle {
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::VISIBLE,
            transform: Transform::from_translation(Vec3::new(CHUNK_SIZE.x / 2.0, CHUNK_SIZE.y / 2.0, 0.0)),
            ..default()
        },
        Anchor::BottomLeft,
        Chunk,
    )).id();

    let brick_start_translation = Vec3::new((-CHUNK_SIZE.x+BRICK_SIZE.x + GAP_BETWEEN_BRICKS)/ 2.0,0.0,0.0);
    for x in 0..11 {
        let brick_translation = Vec3::new(
             brick_start_translation.x + (x as f32) * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS), 0.0,0.0);
        let brick = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: BRICK_COLOR,
                    ..default()
                },
                transform: Transform::from_translation(brick_translation).with_scale(BRICK_SIZE),
                ..default()
            },
            Brick,
            Collilder(CollilderType::BRICK),
        )).id();
    
        commands.entity(chunk).add_child(brick);
    }
    

}

fn move_paddle(
    keyboard_inpit: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Paddle>>
) {
    let mut paddle_velocity = query.single_mut();

    if keyboard_inpit.pressed(KeyCode::A) {
        paddle_velocity.0 = Vec2::new(-1.0, 0.0);
    } else if keyboard_inpit.pressed(KeyCode::D) {
        paddle_velocity.0 = Vec2::new(1.0, 0.0);
    } else {
        paddle_velocity.0 = Vec2::new(0.0, 0.0);
    }

    paddle_velocity.x *= PADDLE_SPEED;
    

    // let new_paddle_position_x = 
    //     paddle_transform.translation.x + paddle_velocity.0.x;

    // let left_bound = LEFT_EDGE - paddle_transform.scale.x / 2.0;
    // let right_bound = RIGHT_EDGE - paddle_transform.scale.x / 2.0;

    // paddle_transform.translation.x = new_paddle_position_x.clamp(left_bound, right_bound);
}

fn gizmos_system(mut gizmos:Gizmos, query: Query<&Transform, With<Chunk>>) {
    for transform in &query {
        let pos = transform.translation.xy();
        gizmos.rect_2d(pos, 0., CHUNK_SIZE.xy(), Color::WHITE);
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn check_paddle_position_edge(mut query: Query<&mut Transform, With<Paddle>>) {
    let mut paddle_transform = query.single_mut();
    
    let left_bound = LEFT_EDGE + paddle_transform.scale.x / 2.0;
    let right_bound = RIGHT_EDGE - paddle_transform.scale.x / 2.0;

    // println!("{} {} {}", paddle_transform.translation.x, left_bound, right_bound);
    paddle_transform.translation.x = paddle_transform.translation.x.clamp(left_bound, right_bound)
}

fn check_ball_position_edge(
    mut commands: Commands, 
    mut query: Query<(Entity, &Transform, &mut Velocity), With<Ball>>) 
{   
    for (entity, ball_transform,  mut velocity) in &mut query {
        if ball_transform.translation.x + BALL_SIZE.x >= RIGHT_EDGE {
            velocity.x = -velocity.x.abs()
        } else if ball_transform.translation.x - BALL_SIZE.x <= LEFT_EDGE {
            velocity.x = velocity.x.abs()
        }
    
        if ball_transform.translation.y + BALL_SIZE.y >= TOP_EDGE {
            velocity.y = -velocity.y.abs()
        } else if ball_transform.translation.y - BALL_SIZE.y < BOTTOM_EDGE {
            commands.entity(entity).despawn();
        }
    }
}

fn check_collider_paddle(
    paddle_query: Query<&Transform, With<Paddle>>,
    mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>,
) {
    let paddle_transform = paddle_query.single();

    for (ball_transform, mut velocity) in &mut ball_query {
        let collision = collide(
            ball_transform.translation,
            ball_transform.scale.truncate(),
            paddle_transform.translation,
            paddle_transform.scale.truncate(),
        );
        
        if let Some(_collision) = collision {
            // match collision {
                // Collision::Left| Collision::Right | Collision::Top  => {
            let point = ((
                ball_transform.translation.x - (paddle_transform.translation.x - paddle_transform.scale.x / 2.0)
            ) / paddle_transform.scale.x).clamp(0.0, 1.0);

            // println!("point {}", point);
            velocity.x = (point - 0.5) / 0.5 * BALL_SPEED;
            velocity.y = (2.0*BALL_SPEED.powf(2.0) - velocity.x.powf(2.0)).sqrt();
                // },
                // _ => {}
            // }
        }
    }
}