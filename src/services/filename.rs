use regex::Regex;

pub struct FilenameService {
    reserved: Regex,
    win_reserved: Regex,
    outer_dots: Regex,
}

impl Default for FilenameService {
    fn default() -> Self {
        Self {
            reserved: Regex::new("[<>:\"/\\\\|?*\u{0000}-\u{001F}\u{007F}\u{0080}-\u{009F}]+")
                .unwrap(),
            win_reserved: Regex::new("^(con|prn|aux|nul|com\\d|lpt\\d)$").unwrap(),
            outer_dots: Regex::new("^\\.+|\\.+$").unwrap(),
        }
    }
}

impl FilenameService {
    pub fn is_valid(&self, filename: &str) -> bool {
        !self.reserved.is_match(filename)
            && !self.win_reserved.is_match(filename)
            && !self.outer_dots.is_match(filename)
    }
}
