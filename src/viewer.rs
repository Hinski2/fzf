use std::cmp::min;
use std::fs::File;
use std::io::{self, BufRead};

use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::util::as_24_bit_terminal_escaped;
use syntect::highlighting::Style;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| SyntaxSet::load_defaults_newlines());
static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| ThemeSet::load_defaults());


pub struct Viewer {
    search_string: String,
    search_results: Vec<usize>,
    file_content: Vec<String>,
    file_extension: String,
    pub display_start: usize,
}

impl Viewer {
    pub fn add_char(&mut self, chr: char) {
        self.search_string.push(chr);
    }

    pub fn pop_char(&mut self) {
        self.search_string.pop();
    }

    pub fn up_char(&mut self) {
        if self.display_start > 0 {
            self.display_start -= 1;
        }
    }

    pub fn down_char(&mut self) {
        if self.display_start + 1 < self.file_content.len() {
            self.display_start += 1;
        }
    }

    pub fn search(&mut self) {
        // updates search_resluts
        self.search_results.clear();

        for (line_no, line_content) in self.file_content.iter().enumerate() {
            if line_content.contains(&self.search_string) {
                self.search_results.push(line_no);
            }
        }
    }

    pub fn new(file_name: &str) -> io::Result<Self> {
        // read the file
        let file = File::open(file_name)?;
        let reader = io::BufReader::new(file);
        let content: Vec<String> = reader
            .lines()
            .collect::<Result<Vec<_>, _>>()?;
        
        let file_extension = file_name.split('.').last().unwrap_or_else(|| "error: file name doesn't contains extension").to_string();

        Ok(Viewer {
            search_string: String::new(),
            search_results: Vec::new(),
            file_content: content,
            file_extension: file_extension,
            display_start: 0,
        })
    }

    pub fn get_lines(&mut self, start: usize, ammount: usize) -> Vec<String> {
        let end = (start + ammount).min(self.file_content.len() - 1);
        self.display_start = min(self.display_start, self.file_content.len());


        let syntax = SYNTAX_SET.find_syntax_by_extension(&self.file_extension).unwrap();
        let mut h = HighlightLines::new(syntax, &THEME_SET.themes["base16-ocean.dark"]);
        let mut styled_content = Vec::<String>::new();

        for line in &self.file_content[start..end] {
            let ranges: Vec<(Style, &str)> = h.highlight_line(&line, &SYNTAX_SET).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            styled_content.push(escaped); 
        }

        styled_content
    }
}
