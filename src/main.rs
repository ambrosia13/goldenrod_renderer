use env_logger::Env;

mod ecs;
mod game;
mod render;
mod ui;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .filter_module("goldenrod", log::LevelFilter::Info)
        .init();

    game::run();
}
