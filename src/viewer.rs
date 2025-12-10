use std::cmp::min;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter;

use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::util::as_24_bit_terminal_escaped;
use syntect::highlighting::Style;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| SyntaxSet::load_defaults_newlines());
static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| ThemeSet::load_defaults());


pub struct Viewer {
    pub search_string: String,
    pub display_start: usize,
    search_results: Vec<SearchResult>,
    file_content: Vec<String>,
    file_extension: String,
    curr_search_idx: usize,
}

struct SearchResult {
    line_no: usize,
    pos_in_line: usize,
}

impl Viewer {
    pub fn go_to_prev_search(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        
        let m = self.search_results.len();
        self.curr_search_idx = (((self.curr_search_idx - 1) % m) + m) % m;
        self.display_start = self.search_results[self.curr_search_idx].line_no;
    }

    pub fn go_to_next_search(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        self.curr_search_idx = (self.curr_search_idx + 1) % self.search_results.len();
        self.display_start = self.search_results[self.curr_search_idx].line_no;
    }

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

        for (line_idx, line) in self.file_content[self.display_start..self.file_content.len()].iter().enumerate() {
            for (pos_in_line, _) in line.match_indices(&self.search_string).into_iter() {
               self.search_results.push(SearchResult { line_no: line_idx, pos_in_line: pos_in_line }); 
            }
        }
        
        for (line_idx, line) in self.file_content[0..self.display_start].iter().enumerate() {
            for (pos_in_line, _) in line.match_indices(&self.search_string).into_iter() {
               self.search_results.push(SearchResult { line_no: line_idx, pos_in_line: pos_in_line }); 
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
            curr_search_idx: 0,
        })
    }

    pub fn get_lines(&mut self, start: usize, ammount: usize) -> (Vec<String>, bool) {
        let end = (start + ammount).min(self.file_content.len() - 1);
        self.display_start = min(self.display_start, self.file_content.len());
        
        let content = match SYNTAX_SET.find_syntax_by_extension(&self.file_extension) {
            Some(syntax) => {
                let mut h = HighlightLines::new(syntax, &THEME_SET.themes["base16-ocean.dark"]);
                let mut styled_content = Vec::<String>::new();

                for line in &self.file_content[start..end] {
                    let ranges: Vec<(Style, &str)> = h.highlight_line(&line, &SYNTAX_SET).unwrap();
                    let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                    styled_content.push(escaped); 
                }

                (styled_content, true)
            },
            None => {
                let mut plane_content = Vec::<String>::new();
                for line in &self.file_content[start..end] {
                    plane_content.push(line.to_string())
                }

                (plane_content, false)
            }
        };

        return content
    }
}
