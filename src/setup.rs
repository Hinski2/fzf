use std::{env, str::FromStr, path::Path};

#[derive(Debug)]
pub struct Setup {
    pub root_dir: String,
    pub deep: u8,
}

impl Setup {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        Setup::from_args(&args)
    }

    fn from_args(args: &[String]) -> Self {
        let mut setup = Setup {
            root_dir: ".".to_string(),
            deep: u8::MAX,
        };
    
        // setup root_dir 
        if args.len() < 2 {
            panic!("error: you need to add root dir");
        }
        setup.root_dir = args[1].to_string();
        Setup::appropriate_root_path(&setup.root_dir).expect("error: innapropriate root path");

        // setup -d
        if let Some(deep) = Setup::contains_flag_with_val(&args, "-d") {
            setup.deep = deep;
        }

        setup
    }
    
    fn contains_flag_with_val<T: FromStr>(args: &[String], flag: &str) -> Option<T> {
        if let Some(pos) = args.iter().position(|x| x == flag) {
            if let Some(val) = args.get(pos + 1) {
                return val.parse::<T>().ok();
            }
        }

        None
    }

    fn contains_flag_without_val(args: &[String], flag: &str) -> Option<()> {
        match args.iter().any(|a| a == flag) {
            true => Some(()),
            false => None, 
        }
    }

    fn appropriate_root_path(root_path: &str) -> Result<(), ()> {
        Path::new(root_path)
            .is_dir()
            .then_some(())
            .ok_or_else(|| ())
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    #[should_panic(expected = "error: you need to add root dir")]
    fn root_dir_absence() {
        let args = vec![
            "prog".into()
        ];
        Setup::from_args(&args);
    }
    
    #[test]
    #[should_panic(expected = "error: innapropriate root path")]
    fn innapropriate_root_path() {
        let args = vec![
            "prog".into(),
            "innapropriate_path".into(),
        ];

        Setup::from_args(&args);
    }

    #[test]
    fn finds_flag() {
        let args = vec![
            "prog".into(),
            ".".into(),
            "-d".into(),
            "7".into(),
        ];

        assert_eq!(Setup::contains_flag_with_val(&args, "-d"), Some(7));
    }
    
    #[test]
    fn properly_parse_flag_with_missing_val() {
        let args = vec![
            "prog".into(),
            ".".into(),
            "-d".into(),
        ];

        assert_eq!(Setup::contains_flag_with_val::<u8>(&args, "-d"), None);
    }

}
