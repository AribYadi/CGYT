use bevy_ecs::prelude::*;
use macroquad::audio::{
  load_sound,
  play_sound,
  play_sound_once,
  PlaySoundParams,
  Sound,
};
use macroquad::prelude::*;

fn window_conf() -> Conf {
  Conf { window_title: "Game Off 2022".to_string(), ..Default::default() }
}

const PLAYER_WIDTH: f32 = 86.0;
const PLAYER_HEIGHT: f32 = 105.0;
const PLAYER_SPEED: f32 = 160.0;
const PLAYER_SPEED_UP_TIME: f32 = 2.0;
const PLAYER_SPEED_UP_SPEED: f32 = 256.0;
const PLAYER_NO_BOUNCE_TIME: f32 = 6.0;
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
const CAT_SLOWING_WIDTH: f32 = 116.0;
const CAT_SLOWING_HEIGHT: f32 = 104.0;
const CAT_SPEED: f32 = 140.0;
const CAT_DEFENDER_PROXIMITY: f32 = 152.0;
const CAT_SLOWING_PROXIMITY: f32 = 224.0;
const CAT_MAX_DEST: f32 = 140.0;
const CAT_ATTACKER_BOUNCE: f32 = 75.0;
const CAT_DEFENDER_BOUNCE: f32 = 152.0;
const CAT_SLOWING_BOUNCE: f32 = 35.0;
const CAT_SLOWING_MUL: f32 = 0.75;

const OBSTACLE_MANEKI_WIDTH: f32 = 78.0;
const OBSTACLE_MANEKI_HEIGHT: f32 = 115.0;
const OBSTACLE_MANEKI_PROXIMITY: f32 = 192.0;

const FONT_SIZE: u16 = 30;
const UI_BG_COLOR: Color = color_u8!(0, 153, 219, 255);
const UI_FG_COLOR: Color = color_u8!(44, 232, 245, 255);
const UI_BUTTON_OUTLINE: f32 = 5.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
  MainMenu,
  LevelSelect,
  Playing,
}

struct Exit(bool);
struct JustPressedBackButton(bool, f32);
struct Level(usize);

struct TextureManager {
  bounce_1: Sound,
  bounce_2: Sound,
  bounce_3: Sound,
  cat_black: Texture2D,
  cat_grey: Texture2D,
  cat_orange: Texture2D,
  cgyt: Texture2D,
  cobblestone: Texture2D,
  lose: Sound,
  manekineko: Texture2D,
  skull_closed: Texture2D,
  skull_open: Texture2D,
  tongue: Texture2D,
  ui: Sound,
  win: Sound,
  yoster_island: Font,
}

#[derive(Component)]
struct Pathfinder {}

impl Pathfinder {
  fn update_pos(
    &mut self,
    start: &mut Rect,
    speed: f32,
    end: Vec2,
    obstacles: &Query<&Obstacle>,
    bouncing: bool,
  ) {
    let dir = (end - start.point()).normalize_or_zero();

    start.x += speed * dir.x * get_frame_time();
    start.y += speed * dir.y * get_frame_time();

    if dir.x != 0.0 && dir.y != 0.0 && !bouncing {
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
  speed_mul: f32,
  bounce_dest: Vec2,
  bounce_percentage: Option<f32>,
}

impl Player {
  fn new(pos: Vec2, powerup_kind: PowerUpKind) -> (Player,) {
    (Player {
      rect: Rect::new(pos.x, pos.y, PLAYER_WIDTH, PLAYER_HEIGHT),
      dir_x: 0.0,
      stun_timer: 0.0,
      powerup_timer: 0.0,
      powerup_kind,
      powerup_cooldown_timer: 0.0,
      animation_timer: PLAYER_ANIMATION_FPS,
      current_frame: 0,
      speed_mul: 1.0,
      bounce_dest: Vec2::ZERO,
      bounce_percentage: None,
    },)
  }
}

#[derive(PartialEq)]
enum PowerUpKind {
  SpeedUp,
  NoBounce,
}

#[derive(Component)]
struct Tongue {
  rect: Rect,
  dir_x: f32,
}

impl Tongue {
  fn new(pos: Vec2) -> (Tongue, Pathfinder) {
    (
      Tongue { rect: Rect::new(pos.x, pos.y, TONGUE_WIDTH, TONGUE_HEIGHT), dir_x: 0.0 },
      Pathfinder {},
    )
  }
}

#[derive(Component)]
struct Cat {
  rect: Rect,
  dir_x: f32,
  kind: CatKind,
  speed_mul: f32,
  bounce_dest: Vec2,
  bounce_percentage: Option<f32>,
}

impl Cat {
  fn new(pos: Vec2, kind: CatKind) -> (Cat, Pathfinder) {
    (
      Cat {
        rect: match kind {
          CatKind::Attacker => Rect::new(pos.x, pos.y, CAT_ATTACKER_WIDTH, CAT_ATTACKER_HEIGHT),
          CatKind::Defender => Rect::new(pos.x, pos.y, CAT_DEFENDER_WIDTH, CAT_DEFENDER_HEIGHT),
          CatKind::Slowing => Rect::new(pos.x, pos.y, CAT_SLOWING_WIDTH, CAT_SLOWING_HEIGHT),
        },
        dir_x: 0.0,
        kind,
        speed_mul: 1.0,
        bounce_dest: Vec2::ZERO,
        bounce_percentage: None,
      },
      Pathfinder {},
    )
  }
}

#[derive(PartialEq)]
enum CatKind {
  Attacker,
  Defender,
  Slowing,
}

#[derive(Component)]
struct Obstacle {
  rect: Rect,
  kind: ObstacleKind,
}

impl Obstacle {
  fn new(pos: Vec2, kind: ObstacleKind) -> (Obstacle,) {
    (Obstacle {
      rect: match kind {
        ObstacleKind::Maneki => {
          Rect::new(pos.x, pos.y, OBSTACLE_MANEKI_WIDTH, OBSTACLE_MANEKI_HEIGHT)
        },
      },
      kind,
    },)
  }
}

#[derive(PartialEq)]
enum ObstacleKind {
  Maneki,
}

fn darken_background() {
  draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(0, 0, 0, 100));
}

fn draw_ui_button(tm: &Res<TextureManager>, rect: &Rect, txt: &str) {
  draw_rectangle(rect.x, rect.y, rect.w, rect.h, UI_FG_COLOR);
  draw_rectangle(
    rect.x + UI_BUTTON_OUTLINE,
    rect.y + UI_BUTTON_OUTLINE,
    rect.w - UI_BUTTON_OUTLINE * 2.0,
    rect.h - UI_BUTTON_OUTLINE * 2.0,
    UI_BG_COLOR,
  );

  let text_measure = measure_text(txt, Some(tm.yoster_island), FONT_SIZE, 1.0);
  draw_text_ex(
    txt,
    rect.center().x - text_measure.width / 2.0,
    rect.center().y + text_measure.offset_y / 2.0,
    TextParams {
      font: tm.yoster_island,
      font_size: FONT_SIZE,
      color: UI_FG_COLOR,
      ..Default::default()
    },
  );
}

fn main_menu(
  tm: Res<TextureManager>,
  mut exit: ResMut<Exit>,
  mut game_state: ResMut<State<GameState>>,
  just_pressed_back_button: Res<JustPressedBackButton>,
) {
  let mouse_pointer: Vec2 = mouse_position().into();

  draw_texture_ex(
    tm.cgyt,
    (screen_width() - tm.cgyt.width() * 2.0) / 2.0,
    (screen_height() - tm.cgyt.height() * 3.5) / 2.0,
    WHITE,
    DrawTextureParams {
      dest_size: Some(vec2(tm.cgyt.width(), tm.cgyt.height()) * 2.0),
      ..Default::default()
    },
  );

  let play_button = Rect::new(screen_width() / 2.0 - 250.0, screen_height() - 175.0, 500.0, 50.0);
  draw_ui_button(&tm, &play_button, "Play");

  if play_button.contains(mouse_pointer) && is_mouse_button_pressed(MouseButton::Left) {
    play_sound_once(tm.ui);
    let _ = game_state.overwrite_set(GameState::LevelSelect);
  }

  let exit_button = Rect::new(screen_width() / 2.0 - 250.0, screen_height() - 100.0, 500.0, 50.0);
  draw_ui_button(&tm, &exit_button, "Exit");

  #[cfg(not(target_arch = "wasm32"))]
  if exit_button.contains(mouse_pointer)
    && is_mouse_button_pressed(MouseButton::Left)
    && !just_pressed_back_button.0
  {
    play_sound_once(tm.ui);
    *exit = Exit(true);
  }
}

fn level_select(
  tm: Res<TextureManager>,
  mut game_state: ResMut<State<GameState>>,
  mut just_pressed_back_button: ResMut<JustPressedBackButton>,
  mut level: ResMut<Level>,
) {
  let mouse_pointer: Vec2 = mouse_position().into();

  let startx = (screen_width() - 75.0 * 5.0) / 2.0;
  let starty = screen_height() / 4.0 - 50.0;
  for i in 0..2 {
    for j in 0..5 {
      let new_level = i * 5 + j + 1;
      if new_level > 10 {
        break;
      }

      let x = startx + 75.0 * j as f32;
      let y = starty + 75.0 * i as f32;

      let button = Rect::new(x, y, 50.0, 50.0);
      draw_ui_button(&tm, &button, &new_level.to_string());

      if button.contains(mouse_pointer) && is_mouse_button_pressed(MouseButton::Left) {
        play_sound_once(tm.ui);
        level.0 = new_level;
        let _ = game_state.overwrite_set(GameState::Playing);
      }
    }
  }

  let back_button = Rect::new(screen_width() / 2.0 - 250.0, screen_height() - 100.0, 500.0, 50.0);
  draw_ui_button(&tm, &back_button, "Back");

  if back_button.contains(mouse_pointer) && is_mouse_button_pressed(MouseButton::Left) {
    play_sound_once(tm.ui);
    let _ = game_state.overwrite_set(GameState::MainMenu);
    just_pressed_back_button.0 = true;
    just_pressed_back_button.1 = 0.1;
  }
}

fn update_misc(mut just_pressed_back_button: ResMut<JustPressedBackButton>) {
  if just_pressed_back_button.1 > 0.0 {
    just_pressed_back_button.1 -= get_frame_time();
  } else {
    just_pressed_back_button.0 = false;
  }
}

fn despawn_all(mut commands: Commands, entities: Query<Entity>) {
  for entity in &entities {
    commands.entity(entity).despawn();
  }
}

fn spawn_player(mut commands: Commands) {
  commands.spawn_bundle(Player::new(vec2(800.0, 600.0) / 2.0, PowerUpKind::SpeedUp));
}

fn spawn_tongue(mut commands: Commands, level: Res<Level>) {
  match level.0 {
    1 => {
      commands.spawn_bundle(Tongue::new(vec2((800.0 - TONGUE_WIDTH) / 2.0, 50.0)));
    },
    2 => {
      commands.spawn_bundle(Tongue::new(vec2(0.0, 50.0)));
    },
    3 => {
      commands.spawn_bundle(Tongue::new(vec2((800.0 - TONGUE_WIDTH) / 2.0, 550.0)));
    },
    4 => {
      commands.spawn_bundle(Tongue::new(vec2(150.0, (600.0 - TONGUE_WIDTH) / 2.0)));
    },
    5 => {
      commands.spawn_bundle(Tongue::new(vec2((800.0 - TONGUE_WIDTH) / 2.0, 0.0)));
    },
    6 => {
      commands.spawn_bundle(Tongue::new(vec2((800.0 - TONGUE_WIDTH) / 2.0, 600.0 - TONGUE_HEIGHT)));
    },
    7 => {
      commands.spawn_bundle(Tongue::new(vec2((800.0 - TONGUE_WIDTH) / 2.0, 600.0 - TONGUE_HEIGHT)));
    },
    8 => {
      commands.spawn_bundle(Tongue::new(vec2(0.0, 600.0 - TONGUE_HEIGHT)));
    },
    9 => {
      commands.spawn_bundle(Tongue::new(vec2(0.0, (600.0 - TONGUE_HEIGHT) / 2.0)));
    },
    10 => {
      commands.spawn_bundle(Tongue::new(vec2(800.0 - TONGUE_WIDTH, (600.0 - TONGUE_WIDTH) / 2.0)));
    },
    _ => {},
  }
}

fn spawn_cat(mut commands: Commands, level: Res<Level>) {
  match level.0 {
    1 => {},
    2 => {
      commands.spawn_bundle(Cat::new(vec2(800.0 - CAT_ATTACKER_WIDTH, 100.0), CatKind::Attacker));
    },
    3 => {
      commands
        .spawn_bundle(Cat::new(vec2((800.0 - CAT_ATTACKER_WIDTH) / 2.0, 0.0), CatKind::Attacker));
      commands.spawn_bundle(Cat::new(vec2(800.0 - CAT_ATTACKER_WIDTH, 500.0), CatKind::Attacker));
    },
    4 => {
      commands.spawn_bundle(Cat::new(
        vec2(800.0 - CAT_ATTACKER_WIDTH, (600.0 - CAT_ATTACKER_WIDTH) / 2.0),
        CatKind::Attacker,
      ));
    },
    5 => {
      commands
        .spawn_bundle(Cat::new(vec2(100.0, (600.0 - CAT_ATTACKER_WIDTH) / 2.0), CatKind::Attacker));
      commands
        .spawn_bundle(Cat::new(vec2(700.0, (600.0 - CAT_ATTACKER_WIDTH) / 2.0), CatKind::Attacker));
    },
    6 => {
      commands
        .spawn_bundle(Cat::new(vec2(100.0, (600.0 - CAT_DEFENDER_WIDTH) / 2.0), CatKind::Defender));
      commands.spawn_bundle(Cat::new(
        vec2(700.0 - CAT_ATTACKER_WIDTH, (600.0 - CAT_ATTACKER_WIDTH) / 2.0),
        CatKind::Attacker,
      ));
    },
    7 => {
      commands.spawn_bundle(Cat::new(vec2(100.0, 400.0), CatKind::Defender));
      commands.spawn_bundle(Cat::new(
        vec2(700.0 - CAT_ATTACKER_WIDTH, CAT_ATTACKER_WIDTH),
        CatKind::Attacker,
      ));
    },
    8 => {
      commands.spawn_bundle(Cat::new(vec2(800.0 - CAT_SLOWING_WIDTH, 100.0), CatKind::Slowing));
    },
    9 => {
      commands.spawn_bundle(Cat::new(
        vec2((800.0 - CAT_SLOWING_WIDTH) / 2.0 + 100.0, 0.0),
        CatKind::Slowing,
      ));
      commands.spawn_bundle(Cat::new(
        vec2((800.0 - CAT_DEFENDER_WIDTH) / 2.0, 600.0 - CAT_DEFENDER_HEIGHT),
        CatKind::Defender,
      ));
    },
    10 => {
      commands.spawn_bundle(Cat::new(
        vec2((800.0 - CAT_SLOWING_WIDTH) / 2.0 - 100.0, 0.0),
        CatKind::Slowing,
      ));
      commands.spawn_bundle(Cat::new(
        vec2((800.0 - CAT_SLOWING_WIDTH) / 2.0 - 100.0, 600.0 - CAT_SLOWING_HEIGHT),
        CatKind::Slowing,
      ));
      commands.spawn_bundle(Cat::new(vec2(400.0, 0.0), CatKind::Defender));
      commands.spawn_bundle(Cat::new(
        vec2(200.0, (600.0 - CAT_ATTACKER_HEIGHT) / 2.0),
        CatKind::Attacker,
      ));
    },
    _ => {},
  }
}

fn spawn_obstacle(mut commands: Commands, level: Res<Level>) {
  match level.0 {
    1 => {},
    2 => {},
    3 => {},
    4 => {
      commands.spawn_bundle(Obstacle::new(
        vec2(0.0, (600.0 - OBSTACLE_MANEKI_HEIGHT) / 2.0) + CAT_ATTACKER_HEIGHT + 10.0,
        ObstacleKind::Maneki,
      ));
    },
    5 => {
      commands.spawn_bundle(Obstacle::new(
        vec2((800.0 - OBSTACLE_MANEKI_WIDTH) / 2.0 - 150.0, 150.0),
        ObstacleKind::Maneki,
      ));
      commands.spawn_bundle(Obstacle::new(
        vec2((800.0 - OBSTACLE_MANEKI_WIDTH) / 2.0 + 150.0 + OBSTACLE_MANEKI_WIDTH, 150.0),
        ObstacleKind::Maneki,
      ));
    },
    6 => {},
    7 => {
      commands.spawn_bundle(Obstacle::new(vec2(200.0, 200.0), ObstacleKind::Maneki));
      commands.spawn_bundle(Obstacle::new(vec2(200.0, 700.0), ObstacleKind::Maneki));
    },
    8 => {},
    9 => {},
    10 => {
      commands.spawn_bundle(Obstacle::new(vec2(900.0, 100.0), ObstacleKind::Maneki));
      commands.spawn_bundle(Obstacle::new(vec2(1200.0, 550.0), ObstacleKind::Maneki));
    },
    _ => {},
  }
}

fn control_player(mut players: Query<&mut Player>, obstacles: Query<&Obstacle>) {
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
      } * player.speed_mul;

      player.rect.x += speed * x as f32 * get_frame_time();
      player.rect.y += speed * y as f32 * get_frame_time();

      if x != 0 && y != 0 && player.bounce_percentage.is_none() {
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

        if trigger_powerup && player.powerup_cooldown_timer <= 0.0 {
          player.powerup_timer = match player.powerup_kind {
            PowerUpKind::SpeedUp => PLAYER_SPEED_UP_TIME,
            PowerUpKind::NoBounce => PLAYER_NO_BOUNCE_TIME,
          };
          player.powerup_cooldown_timer = PLAYER_POWERUP_COOLDOWN;
        } else if player.powerup_timer <= 0.0 {
          player.powerup_cooldown_timer -= get_frame_time();
        }
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
        player.animation_timer = PLAYER_ANIMATION_FPS / player.speed_mul;
        player.current_frame = 1 - player.current_frame;
      }
    }
  }
}

fn bounce_player(mut players: Query<&mut Player>) {
  for mut player in &mut players {
    if let Some(percentage) = player.bounce_percentage {
      let dest = player.rect.point().lerp(player.bounce_dest, percentage.min(1.0));
      let pos = dest - player.rect.point();
      player.rect.x += pos.x;
      player.rect.y += pos.y;
      if percentage > 1.0 {
        player.bounce_percentage = None;
      } else {
        player.bounce_percentage = Some(percentage + 0.05);
      }
    }
  }
}

fn move_tongue(
  mut tongues: Query<(&mut Tongue, &mut Pathfinder)>,
  cats: Query<&Cat>,
  players: Query<&Player>,
  obstacles: Query<&Obstacle>,
) {
  for (mut tongue, mut pathfinder) in &mut tongues {
    let mut dir = Vec2::ZERO;
    for cat in &cats {
      dir += (cat.rect.point() - tongue.rect.point()).normalize_or_zero();
    }
    if dir == Vec2::ZERO {
      for player in &players {
        dir += (player.rect.point() - tongue.rect.point()).normalize_or_zero();
      }
    }
    dir = Vec2::ZERO - dir.normalize_or_zero();
    tongue.dir_x = dir.x;
    let dest = tongue.rect.point() + dir * TONGUE_MAX_DEST;

    pathfinder.update_pos(&mut tongue.rect, TONGUE_SPEED, dest, &obstacles, false);
  }
}

fn tongue_collision(
  tm: Res<TextureManager>,
  tongues: Query<&Tongue>,
  players: Query<&Player>,
  cats: Query<&Cat>,
  mut game_state: ResMut<State<GameState>>,
) {
  for tongue in &tongues {
    if players.iter().any(|player| player.rect.overlaps(&tongue.rect)) {
      play_sound_once(tm.win);
      let _ = game_state.overwrite_set(GameState::LevelSelect);
    }
    if cats.iter().any(|cat| cat.rect.overlaps(&tongue.rect)) {
      play_sound_once(tm.lose);
      let _ = game_state.overwrite_set(GameState::LevelSelect);
    }
  }
}

fn move_cat(
  mut cats: Query<(&mut Cat, &mut Pathfinder)>,
  tongues: Query<&Tongue>,
  mut players: Query<&mut Player>,
  obstacles: Query<&Obstacle>,
) {
  for mut player in &mut players {
    let mut player_slowed = false;

    for (mut cat, mut pathfinder) in &mut cats {
      let proximity_range = match cat.kind {
        CatKind::Attacker => 0.0,
        CatKind::Defender => CAT_DEFENDER_PROXIMITY,
        CatKind::Slowing => CAT_SLOWING_PROXIMITY,
      };

      let proximity = Rect::new(
        cat.rect.x + cat.rect.w / 2.0 - proximity_range,
        cat.rect.y + cat.rect.h / 2.0 - proximity_range,
        proximity_range * 2.0,
        proximity_range * 2.0,
      );

      let is_player_near = proximity.overlaps(&player.rect);

      match cat.kind {
        CatKind::Attacker => (),
        CatKind::Defender => (),
        CatKind::Slowing => {
          if is_player_near {
            player.speed_mul = CAT_SLOWING_MUL;
            player_slowed = true;
          }
        },
      }

      let target = if cat.kind == CatKind::Defender && is_player_near {
        player.rect
      } else {
        tongues.single().rect
      };

      let dir = (target.point() - cat.rect.point()).normalize_or_zero();
      cat.dir_x = dir.x;
      let dest = cat.rect.point() + dir * CAT_MAX_DEST;

      let speed_mul = cat.speed_mul;
      let bounce_percentage = cat.bounce_percentage;
      pathfinder.update_pos(
        &mut cat.rect,
        CAT_SPEED * speed_mul,
        dest,
        &obstacles,
        bounce_percentage.is_some(),
      );
    }

    if !player_slowed {
      player.speed_mul = 1.0;
    }
  }
}

fn cat_collision(
  tm: Res<TextureManager>,
  mut players: Query<&mut Player>,
  mut cats: Query<&mut Cat>,
) {
  for mut player in &mut players {
    for mut cat in &mut cats {
      if player.rect.overlaps(&cat.rect)
        && !(player.powerup_kind == PowerUpKind::NoBounce && player.powerup_timer > 0.0)
        && player.bounce_percentage.is_none()
      {
        let sound = match rand::gen_range(0, 3) {
          0 => tm.bounce_1,
          1 => tm.bounce_2,
          2 => tm.bounce_3,
          _ => unreachable!(),
        };
        play_sound_once(sound);

        let dir = (player.rect.center() - cat.rect.center()).normalize_or_zero();
        let cat_bounce_amount = match cat.kind {
          CatKind::Attacker => CAT_ATTACKER_BOUNCE,
          CatKind::Defender => CAT_DEFENDER_BOUNCE,
          CatKind::Slowing => CAT_SLOWING_BOUNCE,
        };
        cat.bounce_dest = cat.rect.point() - dir * cat_bounce_amount;
        cat.bounce_percentage = Some(0.0);
        player.bounce_dest = player.rect.point() + dir * cat_bounce_amount;
        player.bounce_percentage = Some(0.0);
      }
    }
  }
}

fn bounce_cat(mut cats: Query<&mut Cat>) {
  for mut cat in &mut cats {
    if let Some(percentage) = cat.bounce_percentage {
      let dest = cat.rect.point().lerp(cat.bounce_dest, percentage.min(1.0));
      let pos = dest - cat.rect.point();
      cat.rect.x += pos.x;
      cat.rect.y += pos.y;
      if percentage > 1.0 {
        cat.bounce_percentage = None;
      } else {
        cat.bounce_percentage = Some(percentage + 0.05);
      }
    }
  }
}

fn obstacle_maneki_update(obstacles: Query<&Obstacle>, mut cats: Query<&mut Cat>) {
  for mut cat in &mut cats {
    let mut cat_slowed = false;

    for obstacle in &obstacles {
      match obstacle.kind {
        ObstacleKind::Maneki => {
          let proximity = Rect::new(
            obstacle.rect.x + obstacle.rect.w / 2.0 - OBSTACLE_MANEKI_PROXIMITY,
            obstacle.rect.y + obstacle.rect.h / 2.0 - OBSTACLE_MANEKI_PROXIMITY,
            OBSTACLE_MANEKI_PROXIMITY * 2.0,
            OBSTACLE_MANEKI_PROXIMITY * 2.0,
          );

          let is_cat_near = proximity.overlaps(&cat.rect);
          if is_cat_near {
            cat.speed_mul = 1.5;
            cat_slowed = true;
          }
        },
      }
    }

    if !cat_slowed {
      cat.speed_mul = 1.0;
    }
  }
}

fn update_camera(mut camera: ResMut<Camera2D>, players: Query<&Player>) {
  *camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen_width(), screen_height()));
  camera.target = players.single().rect.center();
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
        CatKind::Slowing => tm.cat_black,
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

fn draw_obstacle(camera: Res<Camera2D>, tm: Res<TextureManager>, obstacles: Query<&Obstacle>) {
  for obstacle in &obstacles {
    let obstacle_pos = camera.world_to_screen(obstacle.rect.point());
    draw_texture_ex(
      match obstacle.kind {
        ObstacleKind::Maneki => tm.manekineko,
      },
      obstacle_pos.x,
      obstacle_pos.y,
      WHITE,
      DrawTextureParams { dest_size: Some(obstacle.rect.size()), ..Default::default() },
    );
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  let mut world = World::new();
  world.insert_resource(State::new(GameState::MainMenu));
  world.insert_resource(Exit(false));
  world.insert_resource(JustPressedBackButton(false, 0.0));
  world.insert_resource(Level(1));
  world.insert_resource(Camera2D::from_display_rect(Rect::new(
    0.0,
    0.0,
    screen_width(),
    screen_height(),
  )));

  let tm = TextureManager {
    bounce_1: load_sound("res/bounce_1.wav").await.unwrap(),
    bounce_2: load_sound("res/bounce_2.wav").await.unwrap(),
    bounce_3: load_sound("res/bounce_3.wav").await.unwrap(),
    cat_black: load_texture("res/cat_black.png").await.unwrap(),
    cat_grey: load_texture("res/cat_grey.png").await.unwrap(),
    cat_orange: load_texture("res/cat_orange.png").await.unwrap(),
    cgyt: load_texture("res/cgyt.png").await.unwrap(),
    cobblestone: load_texture("res/cobblestone.png").await.unwrap(),
    lose: load_sound("res/lose.wav").await.unwrap(),
    manekineko: load_texture("res/manekineko.png").await.unwrap(),
    skull_closed: load_texture("res/skull_closed.png").await.unwrap(),
    skull_open: load_texture("res/skull_open.png").await.unwrap(),
    tongue: load_texture("res/tongue.png").await.unwrap(),
    ui: load_sound("res/ui.wav").await.unwrap(),
    win: load_sound("res/win.wav").await.unwrap(),
    yoster_island: load_ttf_font("res/yoster-island.ttf").await.unwrap(),
  };

  tm.cat_black.set_filter(FilterMode::Nearest);
  tm.cat_grey.set_filter(FilterMode::Nearest);
  tm.cat_orange.set_filter(FilterMode::Nearest);
  tm.cgyt.set_filter(FilterMode::Nearest);
  tm.cobblestone.set_filter(FilterMode::Nearest);
  tm.manekineko.set_filter(FilterMode::Nearest);
  tm.skull_closed.set_filter(FilterMode::Nearest);
  tm.skull_open.set_filter(FilterMode::Nearest);
  tm.tongue.set_filter(FilterMode::Nearest);

  world.insert_resource(tm);

  let mut schedule = Schedule::default()
    .with_stage("update", SystemStage::single_threaded())
    .with_stage_after("update", "late_update", SystemStage::single_threaded());

  schedule.add_system_set_to_stage("update", State::<GameState>::get_driver());
  schedule.add_system_set_to_stage("late_update", State::<GameState>::get_driver());

  schedule.add_system_set_to_stage(
    "update",
    SystemSet::on_enter(GameState::MainMenu).with_system(despawn_all).with_system(spawn_player),
  );
  schedule.add_system_set_to_stage(
    "update",
    SystemSet::on_update(GameState::MainMenu).with_system(update_misc).with_system(update_camera),
  );
  schedule.add_system_set_to_stage(
    "late_update",
    SystemSet::on_update(GameState::MainMenu)
      .with_system(draw_background.label("background"))
      .with_system(darken_background.label("darken_background").after("background"))
      .with_system(main_menu.after("darken_background")),
  );

  schedule.add_system_set_to_stage(
    "update",
    SystemSet::on_enter(GameState::LevelSelect).with_system(despawn_all).with_system(spawn_player),
  );
  schedule.add_system_set_to_stage(
    "update",
    SystemSet::on_update(GameState::LevelSelect)
      .with_system(update_misc)
      .with_system(update_camera),
  );
  schedule.add_system_set_to_stage(
    "late_update",
    SystemSet::on_update(GameState::LevelSelect)
      .with_system(draw_background.label("background"))
      .with_system(darken_background.label("darken_background").after("background"))
      .with_system(level_select.after("darken_background")),
  );

  schedule.add_system_set_to_stage(
    "late_update",
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
      .with_system(bounce_player)
      .with_system(move_tongue)
      .with_system(tongue_collision)
      .with_system(move_cat)
      .with_system(cat_collision)
      .with_system(bounce_cat)
      .with_system(obstacle_maneki_update)
      .with_system(update_camera),
  );
  schedule.add_system_set_to_stage(
    "late_update",
    SystemSet::on_update(GameState::Playing)
      .with_system(draw_background.label("background"))
      .with_system(draw_player.after("background"))
      .with_system(draw_tongue.after("background"))
      .with_system(draw_cat.after("background"))
      .with_system(draw_obstacle.after("background")),
  );

  play_sound(
    load_sound("res/song.wav").await.unwrap(),
    PlaySoundParams { looped: true, volume: 0.3, ..Default::default() },
  );

  loop {
    clear_background(BLACK);

    schedule.run(&mut world);
    if world.resource::<Exit>().0 {
      break;
    }

    next_frame().await;
  }
}
