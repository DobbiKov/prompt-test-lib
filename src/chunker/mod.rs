//! Module providing helpful functions for working with text adjusting it for the LLMs.
//!
//!

use std::io::{Read, Write};

/// Takes a text, divides it into chunks (each chunk containing at most precised
/// _lines\_per\_chunk_ number of lines) and returns the vector of such chunks
pub fn divide_into_chunks(text: String, lines_per_chunk: usize) -> Vec<String> {
    let mut res = Vec::<String>::new();

    if !text.contains("\n") || lines_per_chunk == 0 {
        return vec![text];
    }

    let parts = text.split("\n");
    let mut temp_res = String::new();
    for (id, part) in parts.enumerate() {
        temp_res.push_str(part);
        temp_res.push('\n');

        if (id > 0 && id % lines_per_chunk == 0) {
            res.push(temp_res);
            temp_res = String::new();
        }
    }
    if !temp_res.is_empty() {
        res.push(temp_res);
    }

    res
}

/// Takes a text into parameter and returns the content written in the `<document>` tag.
pub fn extract_translated_from_response(message: String) -> String {
    if !message.contains("<output>") {
        return String::new();
    }
    let mut res = String::new();
    let mut chunks_iter = message.split("<output>");
    let _ = chunks_iter.next();
    while let Some(chunk) = chunks_iter.next() {
        let mut chunk_string = chunk;

        if chunk_string.contains("</output>") {
            chunk_string = chunk_string.split("</output>").next().unwrap()
        }
        res.push_str(chunk_string);
    }
    res
}

/// Reads file and returns its contents in the String format
pub fn read_string_file(path: &str) -> String {
    let mut contents = String::new();
    let mut file = std::fs::File::open(std::path::PathBuf::from(path)).expect("Couldn't open file");
    let _ = file.read_to_string(&mut contents);
    contents
}
