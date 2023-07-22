mod engine;
mod state;
mod texture;
mod camera;
mod rotation;
mod instancing;

fn main() {
    pollster::block_on(engine::run());
}
