/// Returns the string name of the executable from the source file name
pub fn binary_name(src_str: &str) -> String {
    src_str.strip_suffix(".").unwrap_or(src_str).to_owned()
}

/// Returns the string name of the corresponding midi file from the source file name
pub fn midi_name(src_str: &str) -> String {
    let bn = binary_name(src_str);
    bn + ".mid"
}

pub fn bf_name(src_str: &str) -> String {
    let bn = binary_name(src_str);
    bn + ".bf"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_name_tests() {
        assert_eq!(binary_name("test.mid"), "test");
        assert_eq!(binary_name("path/to/dir/test.bf"), "path/to/dir/test.bf");
        assert_eq!(binary_name("no_suffix"), "no_suffix");
    }

    #[test]
    fn midi_name_tests() {
        assert_eq!(midi_name("test.mid"), "test.mid");
        assert_eq!(midi_name("path/to/dir/test.bf"), "path/to/dir/test.mid");
        assert_eq!(midi_name("no_suffix"), "no_suffix.mid");
    }

    #[test]
    fn bf_name_tests() {
        assert_eq!(bf_name("test.mid"), "test.bf");
        assert_eq!(bf_name("path/to/dir/test.mid"), "path/to/dir/test.bf");
        assert_eq!(bf_name("no_suffix"), "no_suffix.bf");
    }
}
