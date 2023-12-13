

use bevy::{
    prelude::*, sprite::{MaterialMesh2dBundle, collide_aabb::collide, Mesh2dHandle},
};
use bevy::sprite::collide_aabb::Collision;
use rand::Rng;



const PADDLE_SIZE: Vec3 = Vec3::new(80.0, 10.0, 0.0);
const PADDLE_COLOR: Color = Color::WHITE;
const PADDLE_SPEED:f32 = 400.0;

const BRICK_SIZE: Vec3 = Vec3::new(10.0, 10.0,0.0);
const BRICK_COLOR: Color = Color::GREEN;
const GAP_BETWEEN_BRICKS: f32 = 2.0;

const BACKGROUND_COLOR: Color = Color::BLUE;

const RIGHT_EDGE: f32 = 640.0;
const LEFT_EDGE: f32 = -640.0;
const TOP_EDGE: f32 = 360.0;
const BOTTOM_EDGE: f32 = -360.0;

const BALL_COLOR: Color = Color::WHITE;
const BALL_SIZE: Vec3 = Vec3::new(8.0, 8.0, 0.0);
const BALL_SPEED: f32 = 200.0;

const WALL_COLOR: Color = Color::GRAY;

const CHUNK_BRICK_SIZE: Vec2 = Vec2::new(8.0, 8.0);
const CHUNK_SIZE: Vec3 = Vec3::new(CHUNK_BRICK_SIZE.x * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),CHUNK_BRICK_SIZE.y * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),0.0);

#[derive(Resource)]
struct BrickCounter(u16);

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Brick {
    destroy: bool
}

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

#[derive(Resource)]
struct GenBallController {
    timer: Timer,
    ball_count: i32,
    // mesh: Mesh2dHandle,
}

impl GenBallController {
    fn new() -> Self {
        // let mesh = meshes.add(shape::Circle::default().into()).into();
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            ball_count: 0,
            // mesh: mesh,
        }
    }
}


enum ColliderType {
    WALL,
    CHUNK,
    BRICK,
    PADDLE,
}

#[derive(Component, Deref, DerefMut)]
struct Collider(ColliderType);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(BrickCounter(100))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(ShowWindowInfoTimer::new())
        .insert_resource(GenBallController::new())
        .add_systems(Startup, setup)
        // .add_systems(Update, (gen_ball))
        .add_systems(FixedUpdate,
            (move_paddle, apply_velocity,
             check_paddle_position_edge,
            check_ball_position_edge,
            check_collider_paddle,
            check_collider_chunk).chain()
        )
        // .add_systems(Update,(gen_ball))
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
        Collider(ColliderType::PADDLE),
    ));

    // let mut rng = rand::thread_rng();

    //ball
    let mut rng = rand::thread_rng();
    let ball_start_x = paddle_translation.x - PADDLE_SIZE.x / 2.0;
    let ball_start_y = paddle_translation.x + PADDLE_SIZE.x / 2.0;
    let ball_translation = Vec3::new(rng.gen_range(ball_start_x..ball_start_y), paddle_translation.y, 0.0);
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

    let max_chunk_col = (RIGHT_EDGE / CHUNK_SIZE.x).ceil();
    let max_chunk_row = (TOP_EDGE / CHUNK_SIZE.y).ceil();

    for c in 0..max_chunk_col as i32 {
        for r in 0..max_chunk_row as i32 {
            let x = CHUNK_SIZE.x / 2.0 + (CHUNK_SIZE.x * c as f32);
            let y = CHUNK_SIZE.y / 2.0 + (CHUNK_SIZE.y * r as f32);

            let chunk_pos = Vec2::new(x, y);
            spawn_chunk(&mut commands,chunk_pos);

            let chunk_pos = Vec2::new(-x, y);
            spawn_chunk(&mut commands,chunk_pos);
        }
    }
}

fn spawn_chunk(commands: &mut Commands, chunk_pos: Vec2) {
    let chunk_entity = commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(chunk_pos.extend(0.0)),
            ..default()
        },
        Chunk,
        Collider(ColliderType::CHUNK)
    )).id();

    //fill brick
    let start_pos = Vec2::new(
        (-CHUNK_SIZE.x +BRICK_SIZE.x + GAP_BETWEEN_BRICKS) / 2.0, 
        (-CHUNK_SIZE.y + BRICK_SIZE.y + GAP_BETWEEN_BRICKS) / 2.0,
    );
    for brick_col in 0..CHUNK_BRICK_SIZE.x as i32 {
        let bc = brick_col as f32;
        for brick_row in 0..CHUNK_BRICK_SIZE.y as i32 {
            let br = brick_row as f32;
            let brick_pos = Vec2::new(
                start_pos.x + (BRICK_SIZE.x + GAP_BETWEEN_BRICKS) * bc,
                start_pos.y + (BRICK_SIZE.y + GAP_BETWEEN_BRICKS) * br,
            );
            if (chunk_pos.x + brick_pos.x).abs() + BRICK_SIZE.x / 2.0 > RIGHT_EDGE {
                continue
            }
            if (chunk_pos.y + brick_pos.y).abs() + BRICK_SIZE.y / 2.0 > TOP_EDGE {
                continue
            }
            
            if ((chunk_pos.y - CHUNK_SIZE.y / 2.0) / CHUNK_SIZE.y).floor() as i32 == 0  && brick_row == 0{
                let brick = commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: WALL_COLOR,
                            ..default()
                        },
                        // global_transform: GlobalTransform::from(Transform::IDENTITY),
                        transform: Transform::from_translation(brick_pos.extend(0.0)).with_scale(BRICK_SIZE),
                        ..default()
                    },
                    WallBlock,
                    Collider(ColliderType::WALL),
                )).id();
                commands.entity(chunk_entity).add_child(brick);
            } else {
                let brick = commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: BRICK_COLOR,
                            ..default()
                        },
                        // global_transform: GlobalTransform::from(Transform::IDENTITY),
                        transform: Transform::from_translation(brick_pos.extend(0.0)).with_scale(BRICK_SIZE),
                        ..default()
                    },
                    Brick {
                        destroy:false,
                    },
                    Collider(ColliderType::BRICK)
                )).id();
                commands.entity(chunk_entity).add_child(brick);
            }

            
        }
    }
}


fn move_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Paddle>>
) {
    let mut paddle_velocity = query.single_mut();

    if keyboard_input.pressed(KeyCode::A) {
        paddle_velocity.0 = Vec2::new(-1.0, 0.0);
    } else if keyboard_input.pressed(KeyCode::D) {
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
    // println!("delta: {}", time.delta_seconds() * 1000.0);
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
    mut query: Query<(Entity, &Transform, &mut Velocity), With<Ball>>,
    mut controller: ResMut<GenBallController>,
) {
    for (entity, ball_transform,  mut velocity) in &mut query {
        if ball_transform.translation.x + BALL_SIZE.x >= RIGHT_EDGE {
            velocity.x = -velocity.x.abs();
        } else if ball_transform.translation.x - BALL_SIZE.x <= LEFT_EDGE {
            velocity.x = velocity.x.abs();
        }
    
        if ball_transform.translation.y + BALL_SIZE.y >= TOP_EDGE {
            velocity.y = -velocity.y.abs();
        } else if ball_transform.translation.y - BALL_SIZE.y < BOTTOM_EDGE {
            // commands.entity(entity).despawn();
            // controller.ball_count -= 1;
            velocity.y = velocity.y.abs();
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
            let point = ((
                ball_transform.translation.x - (paddle_transform.translation.x - paddle_transform.scale.x / 2.0)
            ) / paddle_transform.scale.x).clamp(0.0, 1.0);

            velocity.x = (point - 0.5) / 0.5 * BALL_SPEED;
            velocity.y = (2.0*BALL_SPEED.powf(2.0) - velocity.x.powf(2.0)).sqrt();
        }
    }
}

fn check_collider_chunk(
    mut commands: Commands,
    mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>,
    chunk_query: Query<(&Transform, &Children), With<Chunk>>,
    mut brick_query: Query<(&GlobalTransform, &Transform, AnyOf<(&mut Brick, &WallBlock)>)>,
) {
    // let start_time = SystemTime::now();
    for (ball_transform, mut ball_velocity) in &mut ball_query {
        for (chunk_transform, children) in &chunk_query {
            let mut collided = false;
            let collision = collide(
                ball_transform.translation,
                ball_transform.scale.truncate(),
                chunk_transform.translation,
                Vec2::new(CHUNK_SIZE.x, CHUNK_SIZE.y),
            );

            if collision.is_none() {
                continue
            }
            // println!("in chunk:{} {}", chunk_transform.translation.x, chunk_transform.translation.y)
            let mut shortest: Option<Entity> = None;
            for &child in children {
                if let Ok(brick_item) = brick_query.get_mut(child) {
                    let (global_transform, _, (_, _)) = brick_item;

                    let distance = ball_transform.translation.distance(global_transform.translation());
                    match shortest {
                        Some(last) => {
                            let (last_global_transform, _, (_, _)) = brick_query.get(last).unwrap();
                            if ball_transform.translation.distance(last_global_transform.translation()) > distance {
                                shortest = Some(child)
                            }
                        }
                        None => {
                            shortest = Some(child);
                        }
                    }
                }
            }

            if shortest.is_none() {
                break;
            }

            let target = shortest.unwrap();
            let (global_transform, transform, (brick_option, _)) = brick_query.get_mut(target).unwrap();
            let scale = transform.scale.truncate();
            // let collision = collide(
            //     ball_transform.translation,
            //     ball_transform.scale.truncate(),
            //     global_transform.translation(),
            //     Vec2::new(scale.x + GAP_BETWEEN_BRICKS, scale.y + GAP_BETWEEN_BRICKS),
            // );

            
            let collision = collide_circle_rect(
                ball_transform.translation.truncate(),
                ball_transform.scale.x,
                global_transform.translation().truncate(),
                Vec2::new(scale.x + GAP_BETWEEN_BRICKS, scale.y + GAP_BETWEEN_BRICKS),
            );

            if collision.is_none() {
                if brick_option.is_none() {
                    if ball_transform.translation.y - BALL_SIZE.y / 2.0 <= 11.0 &&
                        ball_transform.translation.y + BALL_SIZE.y / 2.0 >= 1.0 {
                            println!("go wall translation:{} {}", ball_transform.translation.x, ball_transform.translation.y)
                        }
                }
                continue
            }

            if let Some(mut brick) = brick_option {
                if brick.destroy {
                    continue;
                }

                brick.destroy = true;
                commands.entity(target).despawn();
            } else {
                println!("wall translation:{} {} ball translation: {} {} ball velocity: {} {} collision: {:?}", global_transform.translation().x , global_transform.translation().y,
                ball_transform.translation.x, ball_transform.translation.y, ball_velocity.x, ball_velocity.y, collision)
            }

            // println!("collision brick: {} {}",brick_translation.x, brick_translation.y);
            let mut reflect_x = false;
            let mut reflect_y = false;

            let collision= collision.unwrap();
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => {
                    println!("gothrough!! now:{} {} old:{} {} brick:{} {}",ball_transform.translation.x, ball_transform.translation.y,
                    ball_transform.translation.x + ball_velocity.x, ball_transform.translation.y + ball_velocity.y,
                    global_transform.translation().x, global_transform.translation().y
                    )
                }
            }

            if reflect_x {
                collided = true;
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                collided = true;
                ball_velocity.y = -ball_velocity.y;
            }

            if collided {
                break
            }
        }
    }
    // println!("delta:{}",SystemTime::now().duration_since(start_time).unwrap().as_micros())
}

fn check_collider(
    mut commands: Commands,
    mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>,
    mut brick_query: Query<(Entity,&GlobalTransform, &Transform, AnyOf<(&mut Brick, &WallBlock)>)>,
) {
    // let start_time = SystemTime::now();
    for (ball_transform, mut ball_velocity) in &mut ball_query {
        for (entity, global_transform, transform,(brick_option, _)) in &mut brick_query {
            let collision = collide(
                ball_transform.translation,
                ball_transform.scale.truncate(),
                global_transform.translation(),
                transform.scale.truncate(),
            );

            if collision.is_none() {
                continue
            }

            if let Some(mut brick) = brick_option {
                if brick.destroy {
                    continue;
                }
                brick.destroy = true;
                commands.entity(entity).despawn();
            }

            // println!("collision brick: {} {}",brick_translation.x, brick_translation.y);
            let mut reflect_x = false;
            let mut reflect_y = false;

            let collision= collision.unwrap();
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => {
                    println!("gothrough!! now:{} {} old:{} {} brick:{} {}",ball_transform.translation.x, ball_transform.translation.y,
                    ball_transform.translation.x + ball_velocity.x, ball_transform.translation.y + ball_velocity.y,
                    global_transform.translation().x, global_transform.translation().y
                    )
                }
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }

            break
        }
    }
    // println!("delta:{}",SystemTime::now().duration_since(start_time).unwrap().as_micros())
}

fn gen_ball(
    mut commands: Commands,
    time: Res<Time>,
    mut controller: ResMut<GenBallController>,
    ball_query: Query<(&Transform, &Velocity), With<Ball>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if controller.ball_count < 1000 && controller.timer.tick(time.delta()).just_finished() {
        let mesh_handler: Mesh2dHandle = meshes.add(shape::Circle::default().into()).into();
        let material_handler = materials.add(ColorMaterial::from(BALL_COLOR));
        let mut rng = rand::thread_rng();
        for (transform, ball_velocity) in &ball_query {
            for _ in 0..3 {
                let velocity_x = rng.gen_range(-BALL_SPEED..BALL_SPEED);
                let mut velocity_y = (2.0*BALL_SPEED.powf(2.0) - velocity_x.abs().powf(2.0)).sqrt();
                if ball_velocity.y < 0.0 {
                    velocity_y = -velocity_y
                }
                // if rng.gen::<f32>() > 0.5 {
                    // velocity_y = -velocity_y;
                // }
                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: mesh_handler.clone(),
                        material: material_handler.clone(),
                        transform: Transform::from_translation(transform.translation).with_scale(transform.scale),
                        ..default()
                    },
                    Ball,
                    Velocity(Vec2::new(velocity_x, velocity_y)),
                ));
            }
            controller.ball_count += 3;
            if controller.ball_count >= 1000 {
                break
            }
        }
    }
}

fn collide_circle_rect(circle: Vec2, radius: f32, rect: Vec2, rect_size: Vec2) -> Option<Collision> {
    let nearest_x;
    let nearest_y;
    if circle.x < rect.x - rect_size.x * 0.5 {
        nearest_x = rect.x - rect_size.x * 0.5;
    } else if circle.x > rect.x + rect_size.x * 0.5 {
        nearest_x = rect.x + rect_size.x * 0.5;
    } else {
        nearest_x = circle.x;
    }

    if circle.y < rect.y - rect_size.y * 0.5 {
        nearest_y = rect.y - rect_size.y * 0.5
    } else if circle.y > rect.y + rect_size.y * 0.5 {
        nearest_y = rect.y + rect_size.y * 0.5
    } else {
        nearest_y = circle.y;
    }

    let distance = ((nearest_x - circle.x).powf(2.0) + (nearest_y - circle.y).powf(2.0)).sqrt();

    if distance <= radius {
        if nearest_x == rect.x - rect_size.x * 0.5 {
            return Some(Collision::Left)
        } else if nearest_x == rect.x + rect_size.x*0.5 {
            return Some(Collision::Right)
        } else if nearest_y == rect.y - rect_size.y*0.5 {
            return Some(Collision::Bottom)
        } else if nearest_y == rect.y+ rect_size.y*0.5{
            return Some(Collision::Top)
        }
    }

    None
}