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

    res
}
