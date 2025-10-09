use std::{cmp::{max, min}, fs::{self, File}, io::{self, BufRead, BufReader}, path::Path, str};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind, KeyCode};
use ratatui::{
    buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, widgets::{Block, Borders, List, ListItem, Paragraph, Widget}, DefaultTerminal, Frame
};
use ansi_to_tui::IntoText;

use crate::{engine::Engine, viewer};
use crate::viewer::Viewer;


#[derive(PartialEq)]
enum AppMode {
    Left,
    Right(ViewerMode),
}

#[derive(PartialEq)]
enum ViewerMode {
    Normal,
    Search,
}

pub struct App {
    search_string: String,
    engine: Engine,
    exit: bool,
    selected_item_number: usize, 
    selected_item_name: String,

    app_mode: AppMode,
    viewer: Option<Viewer>,
    update_viewer: bool,
}

impl App {
    pub fn new(engine: Engine) -> Self {
        App {search_string: String::new(),
            engine: engine,
            exit: false,
            selected_item_number: 0,
            selected_item_name: String::new(),
            app_mode: AppMode::Left,
            viewer: None,
            update_viewer: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Backspace if self.app_mode == AppMode::Left => self.pop_char(),
            KeyCode::Backspace  => self.viewer.as_mut().unwrap().pop_char(),

            KeyCode::Char(chr) if self.app_mode == AppMode::Left => self.add_char(chr),
            KeyCode::Char(chr) => self.viewer.as_mut().unwrap().add_char(chr),

            KeyCode::Up if self.app_mode == AppMode::Left => self.up_char(),
            KeyCode::Up => self.viewer.as_mut().unwrap().up_char(),

            KeyCode::Down if self.app_mode == AppMode::Left => self.down_char(),
            KeyCode::Down => self.viewer.as_mut().unwrap().down_char(),

            KeyCode::Tab => self.switch_app_mode(),
            KeyCode::Esc => self.exit(),
            _ => {}
        }
    }

    fn switch_app_mode(&mut self) {
        self.app_mode = if self.app_mode == AppMode::Left {AppMode::Right(ViewerMode::Normal)} else {AppMode::Left};
    }
    
    fn up_char(&mut self) {
        if self.selected_item_number + 1 < self.engine.results_size() {
            self.selected_item_number += 1; 
            self.update_viewer = true;
        }
    }

    fn down_char(&mut self) {
        if self.selected_item_number > 0 {
            self.selected_item_number -= 1;
            self.update_viewer = true;
        }
    }

    fn pop_char(&mut self) {
        self.search_string.pop();
        self.engine.pop_char();
    }

    fn add_char(&mut self, chr: char) {
        self.search_string.push(chr);
        self.engine.push_char(chr);
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_list_area(&mut self, list_area: &Rect, buf: &mut Buffer) {
        // get results fro engine
        let h = max(0, list_area.height - 2) as usize;
        let mut items_string: Vec<String> = self.engine.get_items(h);
        
        // update data
        if items_string.is_empty() {
            return;
        }

        self.selected_item_number = min(self.selected_item_number, items_string.len() - 1);
        self.selected_item_name = items_string[self.selected_item_number].clone();


        // draw top empty lines
        for _ in 0..(h - items_string.len()) {
            items_string.push(String::new()); 
        }

        let items: Vec<ListItem> = items_string
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, s)| {
                let mut item = ListItem::new(s);
                if i == h - self.selected_item_number - 1 {
                    item = item.style(Style::default().fg(Color::Yellow));
                }
                item
            })
            .collect();

        let list = List::new(items) 
            .block(
                Block::bordered()
                    .title("Files")
                    .border_style(Style::default().fg(Color::White))
            );

        list.render(*list_area, buf);
    }

    fn is_file_utf8(path: &str) -> io::Result<bool> {
        let bytes = fs::read(path)?;
        Ok(str::from_utf8(&bytes).is_ok())
    }

    fn handle_right_area(&mut self, area: &Rect, buf: &mut Buffer) -> io::Result<()> {
        if Path::new(&self.selected_item_name).is_dir() {
            return Ok(())
        }

         if !Self::is_file_utf8(&self.selected_item_name)? {
            return Ok(());
        }

        if self.update_viewer {
            self.viewer = Some(Viewer::new(&self.selected_item_name).unwrap());
            self.update_viewer = false;
        }

        if let None = self.viewer {
            self.update_viewer = true;
            return Ok(())
        }

        // get content
        let h = area.height as usize;
        let start = self.viewer.as_mut().unwrap().display_start;

        let lines = self.viewer
            .as_mut()
            .unwrap()
            .get_lines(start, h);

        let items: Vec<ListItem> = lines 
            .into_iter()
            .map(|ansi_line| ListItem::new(ansi_line.into_text().unwrap()))
            .collect();
        
        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("file content")
                    .border_style(Style::default().fg(Color::White))
            );

        list.render(*area, buf);

        Ok(())
    }
}

impl Widget for &mut App {

    fn render(self, area:Rect, buf: &mut Buffer) {
        // split page in half
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(area);
    
        // split left area
        let [list_area, input_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .areas(left);
        
        // fill list_area
        App::handle_list_area(self, &list_area, buf);

        // fill input area
        let content = match self.app_mode {
            AppMode::Left => self.search_string.as_str(),
            AppMode::Right(_) => self.search_string.as_str(),
        };

        let input = Paragraph::new(content)
            .style(Style::default().fg(Color::Blue))
            .block(
                Block::bordered()
                .title("Input")
                .border_style(Style::default().fg(Color::White))
            );
        input.render(input_area, buf);

        // fill right area
        if let Err(e) = App::handle_right_area(self, &right ,buf) {
            panic!("Error: {e}")
        };


        // render borders
        let block = |color: Color| {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
        };

        block(Color::White).render(input_area, buf);
        match self.app_mode {
            AppMode::Left => {
                block(Color::Blue).render(list_area, buf);
                block(Color::White).render(right, buf);
            }
            AppMode::Right(_) => {
                block(Color::White).render(list_area, buf);
                block(Color::Blue).render(right, buf);
            }
        }
    }
}
