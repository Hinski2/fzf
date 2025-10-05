use fzf::setup::Setup;
use fzf::engine::{Engine, SearchResult};

fn main() {
    // get env ars
    let setup = Setup::new();
    let engine = Engine::new(setup);
}
