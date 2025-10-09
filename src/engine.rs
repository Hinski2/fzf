use std::{cmp::min, fs, io, path::Path};
use crate::setup::Setup;
use crate::viewer::Viewer;

pub struct Engine {
    setup: Setup,
    base_layer: Vec<String>,
    search_layers: Vec<Vec<SearchResult>>,
}

pub struct SearchResult {
    pub file_id: usize,
    pub search_start: u16, // idx where we can start new search
    pub first_occ: usize,
}



impl Engine {
    pub fn results_size(&self) -> usize {
        self.search_layers.last().unwrap().len()
    }

    pub fn new(setup: Setup) -> Self {
        let mut engine = Engine {
            setup: setup,
            base_layer: Vec::new(),
            search_layers: Vec::new(), 
        };
        
        // create base layer
        let root_dir = engine.setup.root_dir.clone();
        if let Err(e) = engine.find_all_files(Path::new(&root_dir), 0) {
            panic!("error: {e}");
        }

        // make first search_layer for ""
        engine.search_layers.push(Vec::new());
        for id in 0..engine.base_layer.len() {
            engine.search_layers[0].push(SearchResult { file_id: id, search_start: 0, first_occ: usize::MAX });
        }

        engine
    }

    fn find_all_files(&mut self, path: &Path, deep: u8) -> io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_name = entry.path().to_string_lossy().to_string();
            self.base_layer.push(file_name);

            if entry.file_type()?.is_dir() && deep + 1 < self.setup.deep {
                self.find_all_files(&entry.path(), deep + 1)?;
            }
        }

        Ok(())
    }

    fn create_new_layer(&self, chr: char) -> Vec<SearchResult> {
        // creates a new layer after adding new char

        let mut new_layer: Vec<SearchResult> = Vec::new();
        if let Some(layer) = self.search_layers.last() {
            for element in layer {
                let file_id = element.file_id as usize;
                let start = element.search_start as usize;

                if let Some(rel_pos) = self.base_layer[file_id][start..].find(chr) {
                    new_layer.push(SearchResult { file_id: element.file_id, search_start: element.search_start + rel_pos as u16 + 1, first_occ: min(element.first_occ, element.search_start as usize + rel_pos )});
                }
            }
        }

        new_layer
    }

    pub fn get_items(&self, no_items: usize) -> Vec<String> {
        // collects top no_items paths strings 

        let mut names = Vec::<String>::new();

        if let Some(layer) = self.search_layers.last() {
            for search_result in layer {
                if names.len() + 1 > no_items {
                    break;
                }
                names.push(self.base_layer[search_result.file_id as usize].clone());
            }    
        };

        names
    }

    pub fn push_char(&mut self, chr: char) {
        let mut layer = self.create_new_layer(chr); 

        layer.sort_by(|a, b| {
            let len_a = a.search_start as usize - a.first_occ;
            let len_b = b.search_start as usize - b.first_occ;

            if len_a == len_b {
                self.base_layer[a.file_id].len().cmp(&self.base_layer[b.file_id].len())
            } else {
                len_a.cmp(&len_b)
            }
        });

        self.search_layers.push(layer);
    }

    pub fn pop_char(&mut self) {
        if self.search_layers.len() > 1 {
            self.search_layers.pop();
        }
    }
}
