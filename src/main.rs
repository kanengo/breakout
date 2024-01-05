mod collide;
mod json_plugin;

use bevy::{
    prelude::*, sprite::{MaterialMesh2dBundle, collide_aabb::collide, Mesh2dHandle}, input::mouse::MouseMotion, utils::{HashMap}, transform, ecs::world, window::{PrimaryWindow, WindowResolution},
};
use bevy::sprite::collide_aabb::Collision;
use json_plugin::JsonAssetPlugin;
use rand::Rng;
use serde::{Serialize, Deserialize};

const SCREEN_SIZE:(f32, f32) = (720.0, 960.0);
const EDGE_SIZE:(f32, f32) = (680.0, 900.0);

const PADDLE_SIZE: Vec3 = Vec3::new(80.0, 10.0, 0.0);
const PADDLE_COLOR: Color = Color::WHITE;
const PADDLE_SPEED:f32 = 400.0;

const BRICK_SIZE: Vec3 = Vec3::new(10.0, 10.0,0.0);
const BRICK_COLOR: Color = Color::GREEN;
const GAP_BETWEEN_BRICKS: f32 = 2.0;

const BACKGROUND_COLOR: Color = Color::rgb(35.0/255.0, 35.0/255.0, 105.0/255.0);
const EDGE_COLOR: Color = Color::rgb(25.0/255.0, 25.0/255.0, 72.0/255.0);


const RIGHT_EDGE: f32 = EDGE_SIZE.0 / 2.0;
const LEFT_EDGE: f32 = -RIGHT_EDGE;
const TOP_EDGE: f32 = EDGE_SIZE.1 / 2.0;
const BOTTOM_EDGE: f32 = -TOP_EDGE;

const BALL_COLOR: Color = Color::WHITE;
// const BALL_SIZE: Vec3 = Vec3::new(8.0, 8.0, 0.0);
const BALL_SPEED: f32 = 200.0;
const BALL_RADIUS: f32 = 4.0;

const WALL_COLOR: Color = Color::GRAY;

const BREAKOUT_COUNT_PER_REWARD: i32 = 5;

const REWARD_SIZE: Vec2 = Vec2::new(20.0, 35.0);

const CHUNK_BRICK_SIZE: Vec2 = Vec2::new(8.0, 8.0);
const CHUNK_SIZE: Vec3 = Vec3::new(CHUNK_BRICK_SIZE.x * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),CHUNK_BRICK_SIZE.y * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),0.0);

const MAX_BALL_COUNT: i32 = 5000;
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
struct ChunkV2 {
    bricks: HashMap<Entity,u8>,
}

#[derive(Component)]
struct WallBlock;

#[derive(Component, Clone, Copy, Debug)]
struct RewardBrick {
    reward_type: i32,
    reward_param: i32,
}

#[derive(Bundle)]
struct RewardBundle {
    sprite: SpriteBundle,
    reward: RewardBrick,
    velocity: Velocity,
}

impl RewardBundle {
    fn new(pos: Vec2, reward: RewardBrick, texture: Handle<Image>) -> Self{
        Self {
            sprite: SpriteBundle {
                transform: Transform::from_translation(pos.extend(0.0)),
                texture,
                ..default()
            },
            reward,
            velocity: Velocity(Vec2::new(0.0, -200.0)),
        }
    }
}

#[derive(Resource, Default)]
struct ShowWindowInfoTimer(Timer);

impl ShowWindowInfoTimer {
    fn new() -> Self {
        Self(Timer::from_seconds(3.0,TimerMode::Repeating))
    }
}

#[derive(Event, Default)]
struct CollisionEvent(Vec2);

#[derive(Event, Clone, Copy)]
struct GenRewardEvent(Vec2, i32, i32);

#[derive(Event, Deref, Debug, Clone, Copy)]
struct ReceiveRewardEvent(RewardBrick);

#[derive(Resource)]
struct GenBallController {
    timer: Timer,
    ball_count: i32,
    // mesh: Mesh2dHandle,
}

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Resource, Default)]
struct Score {
    val: i32,
    last_reward_val: i32,
    ball_count: i32,
    last_reward_time: HashMap<i32,f32>,
}

impl Score {
    fn new() -> Self {
       Self {
           last_reward_time: HashMap::new(),
           ..default()
       }
    }
}

impl GenBallController {
    fn new() -> Self {
        // let mesh = meshes.add(shape::Circle::default().into()).into();
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            ball_count: 1,
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

#[derive(Default, Serialize, Deserialize,Debug)]
struct BrickData {
   brick_type: u8,
   color: Color,
   pos: Vec2,
}

#[derive(Serialize, Deserialize, Asset, TypePath,Debug)]
struct Level {
    bricks: Vec<BrickData>,
}
#[derive(Resource)]
struct LevelHandler(Handle<Level>);

#[derive(Debug,Clone, Copy,Default,Eq,PartialEq,Hash,States)]
enum AppState {
    #[default]
    Loading,
    Level,
}

#[derive(Resource,Default, Deref, DerefMut)]
struct CursorWorldCoords(Vec2);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "breakout".into(),
                    resolution: WindowResolution::new(SCREEN_SIZE.0, SCREEN_SIZE.1),
                    ..default()
                 }),
                ..default()
            }),
            JsonAssetPlugin::<Level>::new(&["json"])
        ))
        .add_state::<AppState>()
        .init_resource::<CursorWorldCoords>()
        .insert_resource(BrickCounter(100))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(ShowWindowInfoTimer::new())
        .insert_resource(GenBallController::new())
        .insert_resource(Score::new())
        .add_event::<CollisionEvent>()
        .add_event::<GenRewardEvent>()
        .add_event::<ReceiveRewardEvent>()
        .add_systems(Startup, (
            load_level,
            setup,
        ))
        .add_systems(Update, spawn_level.run_if(in_state(AppState::Loading)))
        .add_systems(Update,(
            cursor_to_world_system,
            check_ball_out_range,
            read_collision_events,
            read_gen_reward_events,
            read_receive_reward_events,
            show_info,
            // draw_chunk_rect,
            // print_mouse_events,
        )
        )
        .add_systems(FixedUpdate,(
            (
            move_paddle,
            apply_velocity,
            check_collider_paddle,
            check_collider_ball,
            ).chain(),
            check_receive_rewards,
        ))
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
        println!("Logical size: {:?} x {:?} {}", logical_width, logical_height, window.scale_factor());

        // The size before scaling:
        let physical_width = window.physical_width();
        let physical_height = window.physical_height();
        println!("physical size: {:?} x {:?} {}", physical_width, physical_height, window.scale_factor());

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

fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut level_handler: ResMut<LevelHandler>,
)  {
    let level = asset_server.load("levels/level_1.json");
    let level_handler = LevelHandler(level);

    commands.insert_resource(level_handler);
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {

    let mut window = q_window.single_mut();
    window.cursor.visible = false;

    //camera
    commands.spawn(Camera2dBundle::default());

    //sounds
    let ball_collision_sound: Handle<AudioSource> = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    commands.spawn(SpriteBundle {
        transform: Transform::from_scale(Vec3::new(EDGE_SIZE.0, EDGE_SIZE.1, -10.0)),
        sprite: Sprite {
            color: EDGE_COLOR,
            ..default()
        },
        ..default()
    });

    //paddle
    let paddle_translation = Vec3::new(0.0, -200.0, 0.0);
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
        Collider(ColliderType::PADDLE),
    ));

    // let mut rng = rand::thread_rng();

    //ball
    let mut rng = rand::thread_rng();
    let ball_start_x = paddle_translation.x - PADDLE_SIZE.x / 2.0;
    let ball_start_y = paddle_translation.x + PADDLE_SIZE.x / 2.0;
    let ball_translation = Vec3::new(rng.gen_range(ball_start_x..ball_start_y), paddle_translation.y, 10.0);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(ball_translation).with_scale(Vec2::new(BALL_RADIUS * 2.0, BALL_RADIUS * 2.0).extend(0.0)),
            ..default()
        },
        Ball,
        Velocity(Vec2::new(BALL_SPEED, BALL_SPEED)),
    ));

    // let max_chunk_col = (RIGHT_EDGE / CHUNK_SIZE.x).ceil();
    // let max_chunk_row = (TOP_EDGE / CHUNK_SIZE.y).ceil();

    // for c in 0..max_chunk_col as i32 {
    //     for r in 0..max_chunk_row as i32 {
    //         let x = CHUNK_SIZE.x / 2.0 + (CHUNK_SIZE.x * c as f32);
    //         let y = CHUNK_SIZE.y / 2.0 + (CHUNK_SIZE.y * r as f32);

    //         let chunk_pos = Vec2::new(x, y);
    //         spawn_chunk(&mut commands,chunk_pos, &mut score);

    //         let chunk_pos = Vec2::new(-x, y);
    //         spawn_chunk(&mut commands,chunk_pos, &mut score);
    //     }
    // }
}

fn spawn_level(
    mut commands: Commands,
    mut levels: ResMut<Assets<Level>>,
    level_handle: Res<LevelHandler>,
    mut state: ResMut<NextState<AppState>>,
){ 
    if let Some(level) = levels.remove(level_handle.0.id()) {
        // println!("level:{:?}", level);
        let chunk_size = Vec2::new(
            (BRICK_SIZE.x + GAP_BETWEEN_BRICKS) * CHUNK_BRICK_SIZE.x,
            (BRICK_SIZE.y + GAP_BETWEEN_BRICKS) * CHUNK_BRICK_SIZE.y,
        );
        let mut chunk_m = HashMap::new();
        for level_brick in &level.bricks {
            let zone: i64;
            if level_brick.pos.x > 0.0 {
                if level_brick.pos.y > 0.0 {
                    zone = 1;
                } else {
                    zone = 4;
                }
            } else {
                if level_brick.pos.y > 0.0 {
                    zone = 2;
                } else {
                    zone = 3;
                }
            }
            let x = ((level_brick.pos.x.abs() / chunk_size.x).floor()) as i64;
            let y = ((level_brick.pos.y.abs() / chunk_size.y).floor()) as i64;

            let index = (zone << 32)  + (x << 16) + y;

            if !chunk_m.contains_key(&index) {
                chunk_m.insert(index, ChunkV2{
                    bricks: HashMap::new(),
                });
            }
            // println!("level brick:{:?}", level_brick);
            let  chunk = chunk_m.get_mut(&index).unwrap();
            let brick_id;
            match level_brick.brick_type {
                0 =>{
                    brick_id = commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: level_brick.color,
                                ..default()
                            },
                            // global_transform: GlobalTransform::from(Transform::IDENTITY),
                            transform: Transform::from_translation(level_brick.pos.extend(0.0)).with_scale(BRICK_SIZE),
                            ..default()
                        },
                        Brick {
                            destroy:false,
                        },
                        Collider(ColliderType::BRICK)
                    )).id();
                },
                1 => {
                    brick_id = commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: level_brick.color,
                                ..default()
                            },
                            // global_transform: GlobalTransform::from(Transform::IDENTITY),
                            transform: Transform::from_translation(level_brick.pos.extend(0.0)).with_scale(BRICK_SIZE),
                            ..default()
                        },
                        WallBlock,
                        Collider(ColliderType::WALL),
                    )).id();
                }
                _ => {
                    continue;
                }
            }
        
            chunk.bricks.insert(brick_id, 1);
        }

        for (&index, chunk) in &chunk_m {
            let zone = index >> 32;
            let mut x = (index >> 16 & ((1 << 16) - 1)) as f32;
            let mut y = (index & ((1 << 16) - 1)) as f32;
          
            x = x * (chunk_size.x) + chunk_size.x / 2.0;
            y = y * (chunk_size.y) + chunk_size.y / 2.0;
           

            match zone {
                1 => {}
                2 => {
                    x = -x;
                }
                3 => {
                    x = -x;
                    y = -y;
                }
                4 => {
                    y = -y;
                }
                _ => {}
            };
            
            // info!("zone:{} x: {} y: {} index:{}", zone, x, y, index);
            
            let chunk_pos = Vec2::new(
               x,y
            );
            info!("chunk_pos: {} x {} y {}", chunk_pos, x, y);
            commands.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(chunk_pos.extend(0.0))
                        .with_scale(Vec3::new(
                            (BRICK_SIZE.x + GAP_BETWEEN_BRICKS) * CHUNK_BRICK_SIZE.x,
                            (BRICK_SIZE.y + GAP_BETWEEN_BRICKS) * CHUNK_BRICK_SIZE.y,
                            0.0
                        )),
                    ..default()
                },
                ChunkV2 {
                    bricks: chunk.bricks.to_owned()
                },
                Collider(ColliderType::CHUNK)
            ));
        }

        state.set(AppState::Level);
    }

    
}


fn draw_chunk_rect(
    mut gizmos: Gizmos,
    chunk_query: Query<&Transform, With<ChunkV2>>
) {
    for transform in &chunk_query {
        gizmos.rect_2d(
            transform.translation.truncate(), 
            0.0, 
            transform.scale.truncate(), 
            Color::WHITE,
        );
    }
}

fn spawn_chunk(commands: &mut Commands, chunk_pos: Vec2, score: &mut ResMut<Score>) {
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
            if  (brick_pos.x + chunk_pos.x).abs() < 40.0 {
                continue;
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
                score.ball_count += 1;
                commands.entity(chunk_entity).add_child(brick);
            }

            
        }
    }
}

fn move_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Paddle>>,
    cursor_world_coords: Res<CursorWorldCoords>,
) {
    let mut paddle_transform = query.single_mut();

    if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
        paddle_transform.translation.x -= PADDLE_SPEED * time.delta_seconds();
    } else if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
        paddle_transform.translation.x += PADDLE_SPEED * time.delta_seconds();
    }

    paddle_transform.translation.x = cursor_world_coords.x;
    

    let left_bound = LEFT_EDGE + paddle_transform.scale.x / 2.0;
    let right_bound = RIGHT_EDGE - paddle_transform.scale.x / 2.0;

    paddle_transform.translation.x = paddle_transform.translation.x.clamp(left_bound, right_bound);
}

fn gizmos_system(mut gizmos:Gizmos, query: Query<&Transform, With<Chunk>>) {
    for transform in &query {
        let pos = transform.translation.xy();
        gizmos.rect_2d(pos, 0., CHUNK_SIZE.xy(), Color::WHITE);
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity), Without<Ball>>, time: Res<Time>) {
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

fn check_ball_out_range(
    mut commands: Commands, 
    query: Query<(Entity, &Transform), With<Ball>>,
    paddle_query: Query<&Transform, With<Paddle>>,
    mut controller: ResMut<GenBallController>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, ball_transform) in &query {
        if ball_transform.translation.y + BALL_RADIUS <= BOTTOM_EDGE {
            commands.entity(entity).despawn();
            controller.ball_count -= 1;

            if controller.ball_count == 0 {
                let paddle_transform = paddle_query.single();
                let mut rng = rand::thread_rng();
                let ball_start_x = paddle_transform.translation.x - PADDLE_SIZE.x / 2.0;
                let ball_start_y = paddle_transform.translation.x + PADDLE_SIZE.x / 2.0;
                let ball_translation = Vec3::new(rng.gen_range(ball_start_x..ball_start_y), paddle_transform.translation.y, 10.0);
                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::default().into()).into(),
                        material: materials.add(ColorMaterial::from(BALL_COLOR)),
                        transform: Transform::from_translation(ball_translation).with_scale(Vec2::new(BALL_RADIUS * 2.0, BALL_RADIUS * 2.0).extend(0.0)),
                        ..default()
                    },
                    Ball,
                    Velocity(Vec2::new(BALL_SPEED, BALL_SPEED)),
                ));
                controller.ball_count += 1;
            }
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

fn check_collider_ball(
    mut commands: Commands,
    mut ball_query: Query<(&mut Transform, &mut Velocity), (With<Ball>,Without<ChunkV2>)>,
    chunk_query: Query<(&Transform, &ChunkV2), (With<ChunkV2>, Without<Ball>)>,
    mut brick_query: Query<(&Transform, AnyOf<(&mut Brick, &WallBlock)>),(Without<Ball>, Without<ChunkV2>)>,
    time: Res<Time>,
    mut collision_events: EventWriter<CollisionEvent>
) {
    // let start_time = SystemTime::now();
    for (mut ball_transform, mut ball_velocity) in &mut ball_query {
        let future_ball_translation = Vec2::new(
            ball_transform.translation.x + ball_velocity.x * time.delta_seconds(),
            ball_transform.translation.y + ball_velocity.y * time.delta_seconds(),
        );
        let check_box_translation = Vec2::new(
            (future_ball_translation.x + ball_transform.translation.x) / 2.0,
            (future_ball_translation.y + ball_transform.translation.y) / 2.0,
        ).extend(0.0);

        let check_box_size = Vec2::new(
            (future_ball_translation.x - ball_transform.translation.x).abs() + BALL_RADIUS * 2.0,
            (future_ball_translation.y - ball_transform.translation.y).abs() + BALL_RADIUS * 2.0,
        );

        let mut collision: Option<(f32, Collision, Entity)> = None;

        //检测chunk是否碰撞
        for (chunk_transform, chunk) in &chunk_query {
            if collide(
                check_box_translation,
                check_box_size,
                chunk_transform.translation,
                Vec2::new(CHUNK_SIZE.x, CHUNK_SIZE.y),
            ).is_none() {
                continue
            }

            
            for &child in chunk.bricks.keys() {
                if let Ok(brick_item) = brick_query.get_mut(child) {
                    let (brick_transform, (brick_option, _)) = brick_item;

                    if let Some(brick) = brick_option {
                        if brick.destroy {
                            continue
                        }
                    }

                    if collide(
                        check_box_translation,
                        check_box_size,
                        brick_transform.translation,
                        Vec2::new(BRICK_SIZE.x + GAP_BETWEEN_BRICKS, BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
                    ).is_none() {
                        continue
                    }
                    // println!("toi before: {} ball:{}",  global_transform.translation(), ball_transform.translation);
                    let toi = collide::time_of_collide_circle_rect(
                        ball_transform.translation.truncate(),
                        ball_transform.scale.x * 0.5,
                        ball_velocity.0,
                        brick_transform.translation.truncate(),
                        Vec2::new(BRICK_SIZE.x + GAP_BETWEEN_BRICKS , BRICK_SIZE.y + GAP_BETWEEN_BRICKS )
                        // brick_transform.scale.truncate(),
                    );

                    if let Some((toi,c)) = toi {
                        if toi <= time.delta_seconds() {
                            // println!("toi middle: {} collision: {:?} {} ball:{}", toi, c, global_transform.translation(), ball_transform.translation);
                            match collision {
                                Some((t, _,_)) => {
                                    if toi < t {
                                        collision = Some((toi, c, child))
                                    }
                                }
                                None => {
                                    collision = Some((toi, c,child))
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut edge_collision =  collide::time_of_collide_circle_rect(
            ball_transform.translation.truncate(),
            BALL_RADIUS,
            ball_velocity.0,
            Vec2::ZERO,
            Vec2::new((LEFT_EDGE - RIGHT_EDGE).abs(), (TOP_EDGE - BOTTOM_EDGE).abs()),
        );

        if let Some((toi,_)) = edge_collision {
            if toi > time.delta_seconds() {
                edge_collision = None;
            } else  {
                if let Some((t, _, _)) = collision {
                    if toi < t {
                        collision = None;
                    } else if toi < t {
                        edge_collision = None;
                    }
                }
            }
        }

        if let Some((toi, collision_type, child)) = collision {
            let (transform,(brick_option, _)) = brick_query.get_mut(child).unwrap();

            if let Some(mut brick) = brick_option {
                brick.destroy = true;
                commands.entity(child).despawn();

                collision_events.send(CollisionEvent(transform.translation.truncate()));
            } else {
                // println!("toi: {} collision: {:?} {} ball:{} v:{} delta:{}", toi, collision_type, gt.translation(), ball_transform.translation, ball_velocity.0, time.delta_seconds());
            }

            ball_transform.translation.x += ball_velocity.x * toi;
            ball_transform.translation.y += ball_velocity.y * toi;

            // println!("collision brick: {} {}",brick_translation.x, brick_translation.y);
            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision_type {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => {

                }
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }

        if let Some((toi, collision_type)) = edge_collision {
            if collision.is_none() {
                ball_transform.translation.x += ball_velocity.x * toi;
                ball_transform.translation.y += ball_velocity.y * toi;
            }

            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision_type {
                Collision::Left => reflect_x = ball_velocity.x< 0.0,
                Collision::Right => reflect_x = ball_velocity.x > 0.0,
                Collision::Top => reflect_y = ball_velocity.y > 0.0,
                Collision::Bottom => edge_collision = None,
                Collision::Inside => {}
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }

        if collision.is_none() && edge_collision.is_none() {
            ball_transform.translation.x += ball_velocity.x * time.delta_seconds();
            ball_transform.translation.y += ball_velocity.y * time.delta_seconds();
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

fn read_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut gen_reward_events: EventWriter<GenRewardEvent>,
    mut score: ResMut<Score>,
    time: Res<Time>,
    sound: Res<CollisionSound>,
) {
    if collision_events.is_empty() {
        return
    }

    let get_score= collision_events.len() as i32;
    if (score.val + get_score - score.last_reward_val) / BREAKOUT_COUNT_PER_REWARD > 0 {
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
        let mut count = 0;
        for event in collision_events.read() {
            count += 1;
            if (score.val + count - score.last_reward_val) / BREAKOUT_COUNT_PER_REWARD > 0{
                score.last_reward_val += BREAKOUT_COUNT_PER_REWARD;
                let r:f32 = rng.gen();
                let reward_type;
                if r > 0.6 {
                    reward_type = 1;
                } else {
                    reward_type = 2;
                }
                let val = score.last_reward_time.get(&reward_type);
                match val {
                    Some(last_tick) => {
                        if time.elapsed_seconds() - last_tick < 5.0 {
                            continue
                        }
                        score.last_reward_time.insert(reward_type, time.elapsed_seconds());
                    }
                    None => {
                        score.last_reward_time.insert(reward_type, time.elapsed_seconds());
                    }
                }
                gen_reward_events.send(GenRewardEvent(event.0, reward_type, 2));
            }
        }
    }

   

    // println!("get score:{}", get_score);
    score.val += get_score;

    collision_events.clear();

    commands.spawn(AudioBundle{
        source: sound.0.clone(),
        settings: PlaybackSettings::DESPAWN,
    });
    
}

fn read_gen_reward_events(
    mut commands: Commands,
    mut gen_reward_events: EventReader<GenRewardEvent>,
    asset_server: Res<AssetServer>,
) {
    if gen_reward_events.is_empty() {
        return;
    }

    for &event in gen_reward_events.read() {
        println!("gen reward: {} {} {}", event.0, event.1, event.2);
        let texture;
        if event.1 == 1 {
            texture = asset_server.load("rewards/reward_1.png")
        } else if event.1 == 2 {
            texture = asset_server.load("rewards/reward_2.png")
        } else {
            continue;
        }
        commands.spawn(RewardBundle::new(event.0, RewardBrick{
            reward_type: event.1,
            reward_param: event.2,
        }, texture));
    }

    gen_reward_events.clear();
}

fn check_receive_rewards(
    mut commands: Commands,
    paddle_query: Query<&Transform, With<Paddle>>,
    reward_query: Query<(&Transform, Entity, &RewardBrick), With<RewardBrick>>,
    mut receive_reward_event: EventWriter<ReceiveRewardEvent>,
) {
    let paddle = paddle_query.single();
    for (&transform, reward_entity, &reward_brick) in &reward_query {
        if collide(transform.translation, REWARD_SIZE, paddle.translation, paddle.scale.truncate()).is_none() {
            continue
        }
        receive_reward_event.send(ReceiveRewardEvent(reward_brick));
        commands.entity(reward_entity).despawn();
    }

}

fn read_receive_reward_events(
    mut commands: Commands,
    mut controller: ResMut<GenBallController>,
    mut receive_reward_event: EventReader<ReceiveRewardEvent>,
    ball_query: Query<(&Transform, &Velocity), With<Ball>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    paddle_query: Query<&Transform, With<Paddle>>,
) {
    if receive_reward_event.is_empty() {
        return;
    }

    for &event in receive_reward_event.read() {
        println!("receive reward event:{:?}", event.0);
        match event.reward_type {
            1 => {
                println!("receive reward 1");
                if controller.ball_count < MAX_BALL_COUNT {
                    let mesh_handler: Mesh2dHandle = meshes.add(shape::Circle::default().into()).into();
                    let material_handler = materials.add(ColorMaterial::from(BALL_COLOR));
                    let mut rng = rand::thread_rng();
                    for (transform, ball_velocity) in &ball_query {
                        for _ in 0..event.reward_param {
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
                        controller.ball_count += event.reward_param;
                        if controller.ball_count >= MAX_BALL_COUNT {
                            break
                        }
                    }
                }
            },
            2 => {
                for _ in 0..event.reward_param {
                    let paddle_transform = paddle_query.single();
                    let mut rng = rand::thread_rng();
                    let ball_start_x = paddle_transform.translation.x - PADDLE_SIZE.x / 2.0;
                    let ball_start_y = paddle_transform.translation.x + PADDLE_SIZE.x / 2.0;
                    let ball_translation = Vec3::new(rng.gen_range(ball_start_x..ball_start_y), paddle_transform.translation.y, 10.0);
                    commands.spawn((
                        MaterialMesh2dBundle {
                            mesh: meshes.add(shape::Circle::default().into()).into(),
                            material: materials.add(ColorMaterial::from(BALL_COLOR)),
                            transform: Transform::from_translation(ball_translation).with_scale(Vec2::new(BALL_RADIUS * 2.0, BALL_RADIUS * 2.0).extend(0.0)),
                            ..default()
                        },
                        Ball,
                        Velocity(Vec2::new(BALL_SPEED, BALL_SPEED)),
                    ));
                    controller.ball_count += 1;
                }
            },
            _ => {}
        }
    }

    receive_reward_event.clear();
}

fn gen_ball(
    mut commands: Commands,
    time: Res<Time>,
    mut controller: ResMut<GenBallController>,
    ball_query: Query<(&Transform, &Velocity), With<Ball>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {

}

fn print_mouse_events(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    for event in mouse_motion_events.read() {
        info!("{:?}", event);
    }

    for event in cursor_moved_events.read() {
        info!("{:?}", event);
    }
}

fn cursor_to_world_system(
    mut cursor_world_coords: ResMut<CursorWorldCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    touches: Res<Touches>,
 ){
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();
    
    let mut cursor_position_opt = None;
    for touch in touches.iter() {
        // info!(
        //     "just pressed touch with id: {:?}, at: {:?}",
        //     touch.id(),
        //     touch.position()
        // );
        cursor_position_opt = Some(touch.position());
    }

    let cursor_position;
    if cursor_position_opt.is_none() {
        if let Some(window_cursor_position) = window.cursor_position() {
            cursor_position = window_cursor_position;
        } else {
            return;
        }
    } else {
        cursor_position = cursor_position_opt.unwrap();
    }
    
   
 
    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
       return;
    };
 
    cursor_world_coords.0 = point;
    // window.cursor.visible = true;

   
    // info!("cursor:{:?}, point:{:?}", cursor_position, point)
 }