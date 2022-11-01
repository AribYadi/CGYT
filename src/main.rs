use macroquad::prelude::*;

fn window_conf() -> Conf {
  Conf { window_title: "Game Off 2022".to_string(), ..Default::default() }
}

#[macroquad::main(window_conf)]
async fn main() {
  loop {
    clear_background(WHITE);

    next_frame().await;
  }
}
