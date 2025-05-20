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

pub fn extract_translated_from_response(message: String) -> String {
    if !message.contains("<document>") {
        return String::new();
    }
    let mut res = String::new();
    let mut chunks_iter = message.split("<document>");
    let _ = chunks_iter.next();
    while let Some(chunk) = chunks_iter.next() {
        let mut chunk_string = chunk;

        if chunk_string.contains("</document>") {
            chunk_string = chunk_string.split("</document>").next().unwrap()
        }
        res.push_str(chunk_string);
    }
    res
}
