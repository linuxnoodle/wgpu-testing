mod engine;
mod state;
mod texture;
mod camera;
mod transformation;
mod instancing;
mod model;
mod resources;

fn main() {
    pollster::block_on(engine::run());
}
