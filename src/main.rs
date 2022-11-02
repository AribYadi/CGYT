use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ShouldRun;
use macroquad::prelude::*;

fn window_conf() -> Conf {
  Conf { window_title: "Game Off 2022".to_string(), ..Default::default() }
}

const PLAYER_SIZE: f32 = 32.0;
const PLAYER_SPEED: f32 = 160.0;

const TOUNGE_SIZE: f32 = 24.0;
const TOUNGE_SPEED: f32 = 120.0;

const CAT_SIZE: f32 = 28.0;
const CAT_SPEED: f32 = 140.0;

#[derive(Component)]
struct Player {
  rect: Rect,
}

#[derive(Component)]
struct Tounge {
  rect: Rect,
}

#[derive(Component)]
struct Cat {
  rect: Rect,
}

fn spawn_player(mut commands: Commands) {
  commands.spawn().insert(Player {
    rect: Rect::new(
      screen_width() / 2.0 - PLAYER_SIZE / 2.0,
      screen_height() / 2.0 - PLAYER_SIZE / 2.0,
      PLAYER_SIZE,
      PLAYER_SIZE,
    ),
  });
}

fn spawn_tounge(mut commands: Commands) {
  commands.spawn().insert(Tounge {
    rect: Rect::new(
      screen_width() / 2.0 - TOUNGE_SIZE / 2.0,
      screen_height() / 2.0 - TOUNGE_SIZE / 2.0,
      TOUNGE_SIZE,
      TOUNGE_SIZE,
    ),
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
  });
  commands.spawn().insert(Cat { rect: Rect::new(0.0, 0.0, CAT_SIZE, CAT_SIZE) });
}

fn control_player(mut players: Query<&mut Player>) {
  let x = (is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)) as i32
    - (is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)) as i32;
  let y = (is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)) as i32
    - (is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)) as i32;

  for mut player in &mut players {
    player.rect.x += PLAYER_SPEED * x as f32 * get_frame_time();
    player.rect.y += PLAYER_SPEED * y as f32 * get_frame_time();
  }
}

fn move_tounge(mut tounges: Query<&mut Tounge>, cats: Query<&Cat>) {
  for mut tounge in &mut tounges {
    let mut dir = Vec2::ZERO;
    for cat in &cats {
      dir += (cat.rect.point() - tounge.rect.point()).normalize();
    }
    dir = Vec2::ZERO - dir.normalize();

    tounge.rect.x += TOUNGE_SPEED * dir.x * get_frame_time();
    tounge.rect.y += TOUNGE_SPEED * dir.y * get_frame_time();
  }
}

fn move_cat(mut cats: Query<&mut Cat>, tounges: Query<&Tounge>) {
  for mut cat in &mut cats {
    let dir = (tounges.single().rect.point() - cat.rect.point()).normalize();
    cat.rect.x += CAT_SPEED * dir.x * get_frame_time();
    cat.rect.y += CAT_SPEED * dir.y * get_frame_time();
  }
}

fn use_camera(mut camera: ResMut<Camera2D>, players: Query<&Player>) {
  camera.target = vec2(0.0, 0.0);
  camera.target = players.single().rect.center();
  set_camera(camera.as_ref());
}

fn draw_player(camera: Res<Camera2D>, players: Query<&Player>) {
  for player in &players {
    let player_pos = camera.world_to_screen(player.rect.point());
    draw_rectangle(player_pos.x, player_pos.y, player.rect.w, player.rect.h, GREEN);
  }
}

fn draw_tounge(camera: Res<Camera2D>, tounges: Query<&Tounge>) {
  for tounge in &tounges {
    let tounge_pos = camera.world_to_screen(tounge.rect.point());
    draw_rectangle(tounge_pos.x, tounge_pos.y, tounge.rect.w, tounge.rect.h, RED);
  }
}

fn draw_cat(camera: Res<Camera2D>, cats: Query<&Cat>) {
  for cat in &cats {
    let cat_pos = camera.world_to_screen(cat.rect.point());
    draw_rectangle(cat_pos.x, cat_pos.y, cat.rect.w, cat.rect.h, YELLOW);
  }
}

fn use_default_camera() { set_default_camera(); }

#[macroquad::main(window_conf)]
async fn main() {
  let mut world = World::new();
  world.insert_resource(Camera2D::from_display_rect(Rect::new(
    0.0,
    0.0,
    screen_width(),
    screen_height(),
  )));

  let mut schedule = Schedule::default()
    .with_stage(
      "start",
      SystemStage::parallel()
        .with_run_criteria(ShouldRun::once)
        .with_system(spawn_player)
        .with_system(spawn_tounge)
        .with_system(spawn_cat),
    )
    .with_stage_after(
      "start",
      "update",
      SystemStage::parallel()
        .with_system(control_player)
        .with_system(move_tounge)
        .with_system(move_cat),
    )
    .with_stage_after(
      "update",
      "late_update",
      SystemStage::single_threaded()
        .with_system(use_camera)
        .with_system(draw_player)
        .with_system(draw_tounge)
        .with_system(draw_cat)
        .with_system(use_default_camera),
    );

  loop {
    clear_background(WHITE);

    schedule.run(&mut world);

    next_frame().await;
  }
}
