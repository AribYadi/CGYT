use bevy_ecs::prelude::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
  Conf { window_title: "Game Off 2022".to_string(), ..Default::default() }
}

const PLAYER_WIDTH: f32 = 86.0;
const PLAYER_HEIGHT: f32 = 105.0;
const PLAYER_SPEED: f32 = 160.0;
const PLAYER_SPEED_UP_TIME: f32 = 2.0;
const PLAYER_SPEED_UP_SPEED: f32 = 256.0;
const PLAYER_NO_STUN_TIME: f32 = 4.0;
const PLAYER_POWERUP_COOLDOWN: f32 = 6.0;
const FIX_COLLISION: f32 = 5.0;
const PLAYER_ANIMATION_FPS: f32 = 1.0 / 4.0;

const TONGUE_WIDTH: f32 = 82.0;
const TONGUE_HEIGHT: f32 = 61.0;
const TONGUE_SPEED: f32 = 120.0;
const TONGUE_MAX_DEST: f32 = 120.0;

const CAT_ATTACKER_WIDTH: f32 = 113.0;
const CAT_ATTACKER_HEIGHT: f32 = 105.0;
const CAT_DEFENDER_WIDTH: f32 = 120.0;
const CAT_DEFENDER_HEIGHT: f32 = 104.0;
const CAT_SPEED: f32 = 140.0;
const CAT_PROXIMITY: f32 = 112.0;
const CAT_MAX_DEST: f32 = 140.0;
const CAT_ATTACKER_STUN_TIME: f32 = 0.2;
const CAT_DEFENDER_STUN_TIME: f32 = 1.0;

const OBSTACLE_SIZE: f32 = 128.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
  Won,
  Playing,
  Lose,
}

struct TextureManager {
  cat_grey: Texture2D,
  cat_orange: Texture2D,
  cobblestone: Texture2D,
  skull_closed: Texture2D,
  skull_open: Texture2D,
  tongue: Texture2D,
}

#[derive(Component)]
struct Pathfinder {}

impl Pathfinder {
  fn update_pos(&mut self, start: &mut Rect, speed: f32, end: Vec2, obstacles: &Query<&Obstacle>) {
    let dir = (end - start.point()).normalize_or_zero();

    start.x += speed * dir.x * get_frame_time();
    start.y += speed * dir.y * get_frame_time();

    for obstacle in obstacles {
      let mut intersectioned = (0.0, 0.0);

      while let Some(intersection) = start.intersect(obstacle.rect) {
        intersectioned = (intersection.w, intersection.h);
        if intersection.w > 0.0 {
          start.x -= FIX_COLLISION * dir.x * get_frame_time();
        }
        if intersection.h > 0.0 {
          start.y -= FIX_COLLISION * dir.y * get_frame_time();
        }
      }

      if (intersectioned.0 - intersectioned.1).abs() < f32::EPSILON {
      } else if intersectioned.0 > intersectioned.1 {
        start.y -= speed * dir.y * get_frame_time();
      } else {
        start.x -= speed * dir.x * get_frame_time();
      }
    }
  }
}

#[derive(Component)]
struct Player {
  rect: Rect,
  dir_x: f32,
  stun_timer: f32,
  powerup_timer: f32,
  powerup_kind: PowerUpKind,
  powerup_cooldown_timer: f32,
  animation_timer: f32,
  current_frame: usize,
}

impl Player {
  fn new(pos: Vec2, powerup_kind: PowerUpKind) -> Player {
    Player {
      rect: Rect::new(pos.x, pos.y, PLAYER_WIDTH, PLAYER_HEIGHT),
      dir_x: 0.0,
      stun_timer: 0.0,
      powerup_timer: 0.0,
      powerup_kind,
      powerup_cooldown_timer: 0.0,
      animation_timer: PLAYER_ANIMATION_FPS,
      current_frame: 0,
    }
  }
}

#[derive(PartialEq)]
enum PowerUpKind {
  SpeedUp,
  NoStun,
}

#[derive(Component)]
struct Tongue {
  rect: Rect,
  dir_x: f32,
}

impl Tongue {
  fn new(pos: Vec2) -> Tongue {
    Tongue { rect: Rect::new(pos.x, pos.y, TONGUE_WIDTH, TONGUE_HEIGHT), dir_x: 0.0 }
  }
}

#[derive(Component)]
struct Cat {
  rect: Rect,
  dir_x: f32,
  kind: CatKind,
}

impl Cat {
  fn new(pos: Vec2, kind: CatKind) -> Cat {
    Cat {
      rect: match kind {
        CatKind::Attacker => Rect::new(pos.x, pos.y, CAT_ATTACKER_WIDTH, CAT_ATTACKER_HEIGHT),
        CatKind::Defender => Rect::new(pos.x, pos.y, CAT_DEFENDER_WIDTH, CAT_DEFENDER_HEIGHT),
      },
      dir_x: 0.0,
      kind,
    }
  }
}

#[derive(PartialEq)]
enum CatKind {
  Attacker,
  Defender,
}

#[derive(Component)]
struct Obstacle {
  rect: Rect,
}

impl Obstacle {
  fn new(pos: Vec2) -> Obstacle {
    Obstacle { rect: Rect::new(pos.x, pos.y, OBSTACLE_SIZE, OBSTACLE_SIZE) }
  }
}

// TODO
fn won(mut game_state: ResMut<State<GameState>>) { let _ = game_state.set(GameState::Playing); }

fn despawn_all(mut commands: Commands, entities: Query<Entity>) {
  for entity in &entities {
    commands.entity(entity).despawn();
  }
}

fn spawn_player(mut commands: Commands) {
  commands
    .spawn()
    .insert(Player::new(vec2(screen_width(), screen_width()) / 2.0, PowerUpKind::SpeedUp));
}

fn spawn_tongue(mut commands: Commands) {
  commands
    .spawn()
    .insert(Tongue::new(vec2(screen_width() - TONGUE_WIDTH / 2.0, 0.0)))
    .insert(Pathfinder {});
}

fn spawn_cat(mut commands: Commands) {
  commands
    .spawn()
    .insert(Cat::new(
      vec2(screen_width() - CAT_ATTACKER_WIDTH / 2.0, screen_height() - CAT_ATTACKER_HEIGHT / 2.0),
      CatKind::Attacker,
    ))
    .insert(Pathfinder {});
  commands.spawn().insert(Cat::new(vec2(0.0, 0.0), CatKind::Attacker)).insert(Pathfinder {});
  commands
    .spawn()
    .insert(Cat::new(vec2(0.0, screen_height() - CAT_DEFENDER_WIDTH / 2.0), CatKind::Defender))
    .insert(Pathfinder {});
}

fn spawn_obstacle(mut _commands: Commands) {
  // commands
  // .spawn()
  // .insert(Obstacle {
  // rect: Rect::new(screen_width() + 100.0, -100.0, OBSTACLE_SIZE,
  // OBSTACLE_SIZE), })
  // .insert(Pathfinder {});
}

fn control_player(
  mut players: Query<&mut Player>,
  mut camera: ResMut<Camera2D>,
  obstacles: Query<&Obstacle>,
) {
  let x = (is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)) as i32
    - (is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)) as i32;
  let y = (is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)) as i32
    - (is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)) as i32;
  let trigger_powerup = is_key_pressed(KeyCode::P) || is_key_down(KeyCode::Q);

  for mut player in &mut players {
    if player.stun_timer <= 0.0 {
      if x != 0 {
        player.dir_x = x as f32;
      }

      let speed = if player.powerup_kind == PowerUpKind::SpeedUp && player.powerup_timer > 0.0 {
        PLAYER_SPEED_UP_SPEED
      } else {
        PLAYER_SPEED
      };

      player.rect.x += speed * x as f32 * get_frame_time();
      player.rect.y += speed * y as f32 * get_frame_time();

      for obstacle in &obstacles {
        let mut intersectioned = (0.0, 0.0);

        while let Some(intersection) = player.rect.intersect(obstacle.rect) {
          intersectioned = (intersection.w, intersection.h);
          if intersection.w > 0.0 {
            player.rect.x -= FIX_COLLISION * x as f32 * get_frame_time();
          }
          if intersection.h > 0.0 {
            player.rect.y -= FIX_COLLISION * y as f32 * get_frame_time();
          }
        }

        if (intersectioned.0 - intersectioned.1).abs() < f32::EPSILON {
        } else if intersectioned.0 > intersectioned.1 {
          player.rect.y -= speed * y as f32 * get_frame_time();
        } else {
          player.rect.x -= speed * x as f32 * get_frame_time();
        }
      }

      camera.target = player.rect.center();

      if trigger_powerup && player.powerup_cooldown_timer <= 0.0 {
        player.powerup_timer = match player.powerup_kind {
          PowerUpKind::SpeedUp => PLAYER_SPEED_UP_TIME,
          PowerUpKind::NoStun => PLAYER_NO_STUN_TIME,
        };
        player.powerup_cooldown_timer = PLAYER_POWERUP_COOLDOWN;
      } else if player.powerup_timer <= 0.0 {
        player.powerup_cooldown_timer -= get_frame_time();
      }
    } else {
      player.stun_timer -= get_frame_time();
    }
    if player.powerup_timer > 0.0 {
      player.powerup_timer -= get_frame_time();
    }
  }
}

fn animate_player(mut players: Query<&mut Player>) {
  for mut player in &mut players {
    if player.stun_timer <= 0.0 {
      player.animation_timer -= get_frame_time();
      if player.animation_timer <= 0.0 {
        player.animation_timer = PLAYER_ANIMATION_FPS;
        player.current_frame = 1 - player.current_frame;
      }
    }
  }
}

fn move_tongue(
  mut tongues: Query<(&mut Tongue, &mut Pathfinder)>,
  cats: Query<&Cat>,
  obstacles: Query<&Obstacle>,
) {
  for (mut tongue, mut pathfinder) in &mut tongues {
    let mut dir = Vec2::ZERO;
    for cat in &cats {
      dir += (cat.rect.point() - tongue.rect.point()).normalize_or_zero();
    }
    dir = Vec2::ZERO - dir.normalize_or_zero();
    tongue.dir_x = dir.x;
    let dest = tongue.rect.point() + dir * TONGUE_MAX_DEST;

    pathfinder.update_pos(&mut tongue.rect, TONGUE_SPEED, dest, &obstacles);
  }
}

fn tongue_collision(
  tongues: Query<&Tongue>,
  players: Query<&Player>,
  cats: Query<&Cat>,
  mut game_state: ResMut<State<GameState>>,
) {
  for tongue in &tongues {
    if players.iter().any(|player| player.rect.overlaps(&tongue.rect)) {
      let _ = game_state.set(GameState::Won);
    }
    if cats.iter().any(|player| player.rect.overlaps(&tongue.rect)) {
      let _ = game_state.set(GameState::Lose);
    }
  }
}

fn move_cat(
  mut cats: Query<(&mut Cat, &mut Pathfinder)>,
  tongues: Query<&Tongue>,
  players: Query<&Player>,
  obstacles: Query<&Obstacle>,
) {
  for (mut cat, mut pathfinder) in &mut cats {
    let proximity = Rect::new(
      cat.rect.x + cat.rect.w / 2.0 - CAT_PROXIMITY,
      cat.rect.y + cat.rect.h / 2.0 - CAT_PROXIMITY,
      CAT_PROXIMITY * 2.0,
      CAT_PROXIMITY * 2.0,
    );
    let player = players.single();

    let is_player_near = proximity.overlaps(&player.rect);

    let target = if cat.kind == CatKind::Defender && is_player_near {
      player.rect
    } else {
      tongues.single().rect
    };

    let dir = (target.point() - cat.rect.point()).normalize_or_zero();
    cat.dir_x = dir.x;
    let dest = cat.rect.point() + dir * CAT_MAX_DEST;

    pathfinder.update_pos(&mut cat.rect, CAT_SPEED, dest, &obstacles);
  }
}

fn cat_collision(
  mut commands: Commands,
  mut players: Query<&mut Player>,
  cats: Query<(Entity, &Cat)>,
) {
  for mut player in &mut players {
    for (cat_entity, cat) in &cats {
      if player.rect.overlaps(&cat.rect) {
        commands.entity(cat_entity).despawn();
        if !(player.powerup_kind == PowerUpKind::NoStun && player.powerup_timer > 0.0) {
          player.stun_timer = match cat.kind {
            CatKind::Attacker => CAT_ATTACKER_STUN_TIME,
            CatKind::Defender => CAT_DEFENDER_STUN_TIME,
          };
        }
      }
    }
  }
}

fn draw_background(camera: Res<Camera2D>, tm: Res<TextureManager>, players: Query<&Player>) {
  for player in &players {
    for i in -1..2 {
      for j in -1..2 {
        for y in 0..screen_height() as usize / 128 + 1 {
          for x in 0..screen_width() as usize / 128 + 1 {
            let pos = camera.world_to_screen(
              vec2(x as f32, y as f32) * 128.0
                + vec2(screen_width(), screen_height())
                  * (player.rect.center() / vec2(screen_width(), screen_height())).floor(),
            );
            draw_texture(
              tm.cobblestone,
              pos.x + j as f32 * screen_width(),
              pos.y + i as f32 * screen_height(),
              WHITE,
            );
          }
        }
      }
    }
  }
}

fn draw_player(camera: Res<Camera2D>, tm: Res<TextureManager>, players: Query<&Player>) {
  for player in &players {
    let player_pos = camera.world_to_screen(player.rect.point());
    draw_texture_ex(
      match player.current_frame {
        0 => tm.skull_open,
        1 => tm.skull_closed,
        _ => unreachable!(),
      },
      player_pos.x,
      player_pos.y,
      WHITE,
      DrawTextureParams {
        dest_size: Some(player.rect.size()),
        flip_x: player.dir_x > 0.0,
        ..Default::default()
      },
    );
  }
}

fn draw_tongue(camera: Res<Camera2D>, tm: Res<TextureManager>, tongues: Query<&Tongue>) {
  for tongue in &tongues {
    let tongue_pos = camera.world_to_screen(tongue.rect.point());
    draw_texture_ex(
      tm.tongue,
      tongue_pos.x,
      tongue_pos.y,
      WHITE,
      DrawTextureParams {
        dest_size: Some(tongue.rect.size()),
        flip_x: tongue.dir_x > 0.0,
        ..Default::default()
      },
    );
  }
}

fn draw_cat(camera: Res<Camera2D>, tm: Res<TextureManager>, cats: Query<&Cat>) {
  for cat in &cats {
    let cat_pos = camera.world_to_screen(cat.rect.point());
    draw_texture_ex(
      match cat.kind {
        CatKind::Attacker => tm.cat_grey,
        CatKind::Defender => tm.cat_orange,
      },
      cat_pos.x,
      cat_pos.y,
      WHITE,
      DrawTextureParams {
        dest_size: Some(cat.rect.size()),
        flip_x: cat.dir_x > 0.0,
        ..Default::default()
      },
    );
  }
}

fn draw_obstacle(camera: Res<Camera2D>, obstacles: Query<&Obstacle>) {
  for obstacle in &obstacles {
    let obstacle_pos = camera.world_to_screen(obstacle.rect.point());
    draw_rectangle(obstacle_pos.x, obstacle_pos.y, obstacle.rect.w, obstacle.rect.h, BROWN);
  }
}

// TODO
fn lose(mut game_state: ResMut<State<GameState>>) { let _ = game_state.set(GameState::Playing); }

#[macroquad::main(window_conf)]
async fn main() {
  let mut world = World::new();
  world.insert_resource(State::new(GameState::Playing));
  world.insert_resource(Camera2D::from_display_rect(Rect::new(
    0.0,
    0.0,
    screen_width(),
    screen_height(),
  )));

  let tm = TextureManager {
    cat_grey: load_texture("res/cat_grey.png").await.unwrap(),
    cat_orange: load_texture("res/cat_orange.png").await.unwrap(),
    cobblestone: load_texture("res/cobblestone.png").await.unwrap(),
    skull_closed: load_texture("res/skull_closed.png").await.unwrap(),
    skull_open: load_texture("res/skull_open.png").await.unwrap(),
    tongue: load_texture("res/tongue.png").await.unwrap(),
  };

  tm.cat_grey.set_filter(FilterMode::Nearest);
  tm.cat_orange.set_filter(FilterMode::Nearest);
  tm.cobblestone.set_filter(FilterMode::Nearest);
  tm.skull_closed.set_filter(FilterMode::Nearest);
  tm.skull_open.set_filter(FilterMode::Nearest);
  tm.tongue.set_filter(FilterMode::Nearest);

  world.insert_resource(tm);

  let mut schedule = Schedule::default()
    .with_stage("update", SystemStage::parallel())
    .with_stage_after("update", "late_update", SystemStage::single_threaded());

  schedule.add_system_set_to_stage("update", State::<GameState>::get_driver());
  schedule.add_system_set_to_stage("late_update", State::<GameState>::get_driver());

  schedule.add_system_set_to_stage("update", SystemSet::on_update(GameState::Won).with_system(won));

  schedule.add_system_set_to_stage(
    "update",
    SystemSet::on_enter(GameState::Playing)
      .with_system(despawn_all)
      .with_system(spawn_player)
      .with_system(spawn_tongue)
      .with_system(spawn_cat)
      .with_system(spawn_obstacle),
  );
  schedule.add_system_set_to_stage(
    "update",
    SystemSet::on_update(GameState::Playing)
      .with_system(control_player)
      .with_system(animate_player)
      .with_system(move_tongue)
      .with_system(tongue_collision)
      .with_system(move_cat)
      .with_system(cat_collision),
  );
  schedule.add_system_set_to_stage(
    "late_update",
    SystemSet::on_update(GameState::Playing)
      .with_system(draw_background.label("background"))
      .with_system(draw_player.after("background"))
      .with_system(draw_tongue.after("background"))
      .with_system(draw_cat.after("background"))
      .with_system(draw_obstacle.after("jbackground")),
  );

  schedule
    .add_system_set_to_stage("update", SystemSet::on_update(GameState::Lose).with_system(lose));

  loop {
    clear_background(BLACK);

    schedule.run(&mut world);

    next_frame().await;
  }
}
