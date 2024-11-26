use macroquad::{
    input,
    prelude::*,
    ui::{hash, root_ui, widgets},
}; // 0.8.

mod simulation;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;

fn window_conf() -> Conf {
    Conf {
        window_resizable: false,
        window_title: "Natural Control".to_owned(),
        window_height: HEIGHT,
        window_width: WIDTH,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut simulation = simulation::Simulation::new(WIDTH as f32, HEIGHT as f32);
    simulation.is_running = true;

    while simulation.is_running {
        simulation.frame();

        next_frame().await
    }
}
