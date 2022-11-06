use bevy_ecs::prelude::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
  Conf { window_title: "Game Off 2022".to_string(), ..Default::default() }
}

const PLAYER_SIZE: f32 = 32.0;
const PLAYER_SPEED: f32 = 160.0;
const PLAYER_SPEED_UP_TIME: f32 = 2.0;
const PLAYER_SPEED_UP_SPEED: f32 = 256.0;
const PLAYER_NO_STUN_TIME: f32 = 4.0;
const PLAYER_POWERUP_COOLDOWN: f32 = 6.0;
const PLAYER_FIX_COLLISION: f32 = 5.0;

const TONGUE_SIZE: f32 = 24.0;
const TONGUE_SPEED: f32 = 120.0;
const TONGUE_MAX_DEST: f32 = 120.0;

const CAT_SIZE: f32 = 28.0;
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

#[derive(Component)]
struct Pathfinder {}

impl Pathfinder {
  fn update_pos(&mut self, start: &mut Rect, speed: f32, end: Vec2) {
    let dir = (end - start.point()).normalize_or_zero();
    start.x += speed * dir.x * get_frame_time();
    start.y += speed * dir.y * get_frame_time();
  }
}

#[derive(Component)]
struct Player {
  rect: Rect,
  stun_timer: f32,
  powerup_timer: f32,
  powerup_kind: PowerUpKind,
  powerup_cooldown_timer: f32,
}

#[derive(PartialEq)]
enum PowerUpKind {
  SpeedUp,
  NoStun,
}

#[derive(Component)]
struct Tongue {
  rect: Rect,
}

#[derive(Component)]
struct Cat {
  rect: Rect,
  kind: CatKind,
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

// TODO
fn won(mut bg_color: ResMut<Color>, mut game_state: ResMut<State<GameState>>) {
  *bg_color = DARKGREEN;
  let _ = game_state.set(GameState::Playing);
}

fn despawn_all(mut commands: Commands, entities: Query<Entity>) {
  for entity in &entities {
    commands.entity(entity).despawn();
  }
}

fn spawn_player(mut commands: Commands) {
  commands.spawn().insert(Player {
    rect: Rect::new(
      screen_width() / 2.0 - PLAYER_SIZE / 2.0,
      screen_height() / 2.0 - PLAYER_SIZE / 2.0,
      PLAYER_SIZE,
      PLAYER_SIZE,
    ),
    stun_timer: 0.0,
    powerup_timer: 0.0,
    powerup_kind: PowerUpKind::NoStun,
    powerup_cooldown_timer: 0.0,
  });
}

fn spawn_tongue(mut commands: Commands) {
  commands
    .spawn()
    .insert(Tongue {
      rect: Rect::new(screen_width() - TONGUE_SIZE / 2.0, 0.0, TONGUE_SIZE, TONGUE_SIZE),
    })
    .insert(Pathfinder {});
}

fn spawn_cat(mut commands: Commands) {
  commands
    .spawn()
    .insert(Cat {
      rect: Rect::new(
        screen_width() - CAT_SIZE / 2.0,
        screen_height() - CAT_SIZE / 2.0,
        CAT_SIZE,
        CAT_SIZE,
      ),
      kind: CatKind::Attacker,
    })
    .insert(Pathfinder {});
  commands
    .spawn()
    .insert(Cat { rect: Rect::new(0.0, 0.0, CAT_SIZE, CAT_SIZE), kind: CatKind::Attacker })
    .insert(Pathfinder {});
  commands
    .spawn()
    .insert(Cat {
      rect: Rect::new(0.0, screen_height() - CAT_SIZE / 2.0, CAT_SIZE, CAT_SIZE),
      kind: CatKind::Defender,
    })
    .insert(Pathfinder {});
}

fn spawn_obstacle(mut commands: Commands) {
  commands
    .spawn()
    .insert(Obstacle {
      rect: Rect::new(screen_width() + 100.0, -100.0, OBSTACLE_SIZE, OBSTACLE_SIZE),
    })
    .insert(Pathfinder {});
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
      let speed = if player.powerup_kind == PowerUpKind::SpeedUp && player.powerup_timer > 0.0 {
        PLAYER_SPEED_UP_SPEED
      } else {
        PLAYER_SPEED
      };

      player.rect.x += speed * x as f32 * get_frame_time();
      player.rect.y += speed * y as f32 * get_frame_time();

      for obstacle in &obstacles {
        while let Some(intersection) = player.rect.intersect(obstacle.rect) {
          if intersection.w > 0.0 {
            player.rect.x -= PLAYER_FIX_COLLISION * x as f32 * get_frame_time();
          }
          if intersection.h > 0.0 {
            player.rect.y -= PLAYER_FIX_COLLISION * y as f32 * get_frame_time();
          }
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

fn move_tongue(mut tongues: Query<(&mut Tongue, &mut Pathfinder)>, cats: Query<&Cat>) {
  for (mut tongue, mut pathfinder) in &mut tongues {
    let mut dir = Vec2::ZERO;
    for cat in &cats {
      dir += (cat.rect.point() - tongue.rect.point()).normalize_or_zero();
    }
    dir = Vec2::ZERO - dir.normalize_or_zero();
    let dest = tongue.rect.point() + dir * TONGUE_MAX_DEST;

    pathfinder.update_pos(&mut tongue.rect, TONGUE_SPEED, dest);
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
) {
  for (mut cat, mut pathfinder) in &mut cats {
    let proximity = Rect::new(
      cat.rect.x + CAT_SIZE / 2.0 - CAT_PROXIMITY,
      cat.rect.y + CAT_SIZE / 2.0 - CAT_PROXIMITY,
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
    let dest = cat.rect.point() + dir * CAT_MAX_DEST;

    pathfinder.update_pos(&mut cat.rect, CAT_SPEED, dest);
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

fn draw_player(camera: Res<Camera2D>, players: Query<&Player>) {
  for player in &players {
    let player_pos = camera.world_to_screen(player.rect.point());
    draw_rectangle(player_pos.x, player_pos.y, player.rect.w, player.rect.h, GREEN);
  }
}

fn draw_tongue(camera: Res<Camera2D>, tongues: Query<&Tongue>) {
  for tongue in &tongues {
    let tongue_pos = camera.world_to_screen(tongue.rect.point());
    draw_rectangle(tongue_pos.x, tongue_pos.y, tongue.rect.w, tongue.rect.h, RED);
  }
}

fn draw_cat(camera: Res<Camera2D>, cats: Query<&Cat>) {
  for cat in &cats {
    let cat_pos = camera.world_to_screen(cat.rect.point());
    draw_rectangle(
      cat_pos.x,
      cat_pos.y,
      cat.rect.w,
      cat.rect.h,
      match cat.kind {
        CatKind::Attacker => YELLOW,
        CatKind::Defender => GRAY,
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
fn lose(mut bg_color: ResMut<Color>, mut game_state: ResMut<State<GameState>>) {
  *bg_color = MAROON;
  let _ = game_state.set(GameState::Playing);
}

#[macroquad::main(window_conf)]
async fn main() {
  let mut world = World::new();
  world.insert_resource(WHITE);
  world.insert_resource(State::new(GameState::Playing));
  world.insert_resource(Camera2D::from_display_rect(Rect::new(
    0.0,
    0.0,
    screen_width(),
    screen_height(),
  )));

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
      .with_system(move_tongue)
      .with_system(tongue_collision)
      .with_system(move_cat)
      .with_system(cat_collision),
  );
  schedule.add_system_set_to_stage(
    "late_update",
    SystemSet::on_update(GameState::Playing)
      .with_system(draw_player)
      .with_system(draw_tongue)
      .with_system(draw_cat)
      .with_system(draw_obstacle),
  );

  schedule
    .add_system_set_to_stage("update", SystemSet::on_update(GameState::Lose).with_system(lose));

  loop {
    clear_background(*world.resource());

    schedule.run(&mut world);

    next_frame().await;
  }
}
