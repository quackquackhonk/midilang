/// Returns the string name of the executable from the source file name
pub fn binary_name(src_str: &str) -> String {
    src_str.strip_suffix('.').unwrap_or(src_str).to_owned()
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
