pub fn chunk_string(s: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_length = 0;

    for line in s.lines() {
        let words = line.split_whitespace().collect::<Vec<&str>>();

        for word in words {
            let word_length = word.chars().count();

            // Check if adding this word would exceed the chunk size
            if current_length + word_length > chunk_size {
                // Push the current chunk and start a new one if the current chunk is not empty
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                    current_chunk.clear();
                }
                current_length = 0;
            }

            // Add the word to the chunk, handle space addition
            if current_length > 0 {
                current_chunk.push(' ');
                current_length += 1; // for the space
            }
            current_chunk.push_str(word);
            current_length += word_length;

            // Manage exactly full chunks
            if current_length == chunk_size {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
                current_length = 0;
            }
        }

        // Add newline at the end of the line if there's any content
        if !current_chunk.is_empty() {
            current_chunk.push('\n');
            current_length += 1;
        }
    }

    // Add the last chunk if not empty
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}
