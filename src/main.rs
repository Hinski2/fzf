use std::io;

use fzf::setup::Setup;
use fzf::engine::Engine;
use fzf::app::App;

fn main() -> io::Result<()> {
    // get env ars
    let setup = Setup::new();
    let engine = Engine::new(setup);

    let mut terminal = ratatui::init();
    let app_result = App::new(engine).run(&mut terminal);
    ratatui::restore();
    app_result
}
