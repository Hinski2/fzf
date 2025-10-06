use std::{cmp::{max, min}, fs::{self, File}, io::{self, BufRead, BufReader}, path::Path, str};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind, KeyCode};
use ratatui::{
    buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, widgets::{block::title, Block, Borders, List, ListItem, Paragraph, Widget}, DefaultTerminal, Frame
};

use crate::engine::{Engine, SearchResult};

pub struct App {
    search_string: String,
    engine: Engine,
    exit: bool,
    selected_item_number: usize, 
    selected_item_name: String,
}

impl App {
    pub fn new(engine: Engine) -> Self {
        App {search_string: String::new(), engine: engine, exit: false, selected_item_number: 0, selected_item_name: String::new()}
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
            KeyCode::Esc => self.exit(),
            KeyCode::Backspace => self.pop_char(),
            KeyCode::Char(chr) => self.add_char(chr),
            KeyCode::Up => self.up_char(),
            KeyCode::Down => self.down_char(),
            _ => {}
        }
    }
    
    fn up_char(&mut self) {
       self.selected_item_number += 1; 
    }

    fn down_char(&mut self) {
        if self.selected_item_number > 0 {
            self.selected_item_number -= 1;
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

    fn handle_right_area(&self, area: &Rect, buf: &mut Buffer) -> io::Result<()> {
        if Path::new(&self.selected_item_name).is_dir() {
            return Ok(())
        }

         if !Self::is_file_utf8(&self.selected_item_name)? {
            return Ok(());
        }

        // get content
        let h = area.height as usize;
        let file = File::open(&self.selected_item_name)?;
        let reader = BufReader::new(file);
    
        let lines: Vec<String> = reader
            .lines()
            .take(h)
            .collect::<Result<_, _>>()?;

        let items: Vec<ListItem> = lines 
            .into_iter()
            .map(|e| ListItem::new(e))
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
        let input = Paragraph::new(self.search_string.as_str())
            .style(Style::default().fg(Color::Blue))
            .block(
                Block::bordered()
                .title("Input")
                .border_style(Style::default().fg(Color::White))
            );

        // fill right area
        if let Err(e) = App::handle_right_area(self, &right ,buf) {
            panic!("Error: {e}")
        };

        input.render(input_area, buf);

        // render borders
        let white_block = || {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
        };

        white_block().render(list_area, buf);
        white_block().render(input_area, buf);
        white_block().render(right, buf);
    }
}
