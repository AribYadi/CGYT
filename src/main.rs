use bevy_ecs::prelude::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
  Conf { window_title: "Game Off 2022".to_string(), ..Default::default() }
}

const PLAYER_SIZE: f32 = 32.0;
const PLAYER_SPEED: f32 = 160.0;

const TONGUE_SIZE: f32 = 24.0;
const TONGUE_SPEED: f32 = 120.0;

const CAT_SIZE: f32 = 28.0;
const CAT_SPEED: f32 = 140.0;
const CAT_PROXIMITY: f32 = 112.0;
const CAT_ATTACKER_STUN_TIME: f32 = 0.2;
const CAT_DEFENDER_STUN_TIME: f32 = 1.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
  Won,
  Playing,
  Lose,
}

#[derive(Component)]
struct Player {
  rect: Rect,
  stun_timer: f32,
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
  });
}

fn spawn_tongue(mut commands: Commands) {
  commands.spawn().insert(Tongue {
    rect: Rect::new(screen_width() - TONGUE_SIZE / 2.0, 0.0, TONGUE_SIZE, TONGUE_SIZE),
  });
}

fn spawn_cat(mut commands: Commands) {
  commands.spawn().insert(Cat {
    rect: Rect::new(
      screen_width() - CAT_SIZE / 2.0,
      screen_height() - CAT_SIZE / 2.0,
      CAT_SIZE,
      CAT_SIZE,
    ),
    kind: CatKind::Attacker,
  });
  commands
    .spawn()
    .insert(Cat { rect: Rect::new(0.0, 0.0, CAT_SIZE, CAT_SIZE), kind: CatKind::Attacker });
  commands.spawn().insert(Cat {
    rect: Rect::new(0.0, screen_height() - CAT_SIZE / 2.0, CAT_SIZE, CAT_SIZE),
    kind: CatKind::Defender,
  });
}

fn control_player(mut players: Query<&mut Player>, mut camera: ResMut<Camera2D>) {
  let x = (is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)) as i32
    - (is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)) as i32;
  let y = (is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)) as i32
    - (is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)) as i32;

  for mut player in &mut players {
    if player.stun_timer <= 0.0 {
      player.rect.x += PLAYER_SPEED * x as f32 * get_frame_time();
      player.rect.y += PLAYER_SPEED * y as f32 * get_frame_time();
      camera.target = player.rect.center();
    } else {
      player.stun_timer -= get_frame_time();
    }
  }
}

fn move_tongue(mut tongues: Query<&mut Tongue>, cats: Query<&Cat>) {
  for mut tongue in &mut tongues {
    let mut dir = Vec2::ZERO;
    for cat in &cats {
      dir += (cat.rect.point() - tongue.rect.point()).normalize_or_zero();
    }
    dir = Vec2::ZERO - dir.normalize_or_zero();

    tongue.rect.x += TONGUE_SPEED * dir.x * get_frame_time();
    tongue.rect.y += TONGUE_SPEED * dir.y * get_frame_time();
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

fn move_cat(mut cats: Query<&mut Cat>, tongues: Query<&Tongue>, players: Query<&Player>) {
  for mut cat in &mut cats {
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
    cat.rect.x += CAT_SPEED * dir.x * get_frame_time();
    cat.rect.y += CAT_SPEED * dir.y * get_frame_time();
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
        player.stun_timer = match cat.kind {
          CatKind::Attacker => CAT_ATTACKER_STUN_TIME,
          CatKind::Defender => CAT_DEFENDER_STUN_TIME,
        };
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
      .with_system(spawn_cat),
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
      .with_system(draw_cat),
  );

  schedule
    .add_system_set_to_stage("update", SystemSet::on_update(GameState::Lose).with_system(lose));

  loop {
    clear_background(*world.resource());

    schedule.run(&mut world);

    next_frame().await;
  }
}
