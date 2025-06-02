use memchr::{memchr, memchr_iter};

pub fn sanitize_r3_locale_file(file: &[u8]) -> Box<[u8]>{
    if !std::str::from_utf8(&file).is_ok(){
        panic!("Invalid UTF-8 characters found!");
    }
    let file_len = file.len();
    let mut temp_file = Vec::with_capacity(file_len);
    let mut last_pos = 0;
    for pos in memchr_iter(b'\r', file) {
        temp_file.extend_from_slice(&file[last_pos..pos]);
        if file.get(pos + 1) == Some(&b'\n') {
            temp_file.push(b'\n');
            last_pos = pos + 2;
        } else {
            temp_file.push(b'\n');
            last_pos = pos + 1;
        }
    }
    temp_file.extend_from_slice(&file[last_pos..]);

    let comment_opening_matches_initial: Vec<usize> = memchr_iter(b'#', &*temp_file).collect();
    let mut comment_closing_matches = Vec::new();
    let mut comment_opening_matches: Vec<usize> = Vec::with_capacity(comment_opening_matches_initial.len()/2);
    for item in &comment_opening_matches_initial{
        if comment_opening_matches_initial.contains(&(item + 1)) && (*item == 0 || !comment_opening_matches_initial.contains(&(item - 1))){
            if let Some(close_pos) = memchr(b'\n', &temp_file[*item..]) {
                comment_opening_matches.push(*item);
                comment_closing_matches.push(Some(close_pos));
            } else {
                comment_opening_matches.push(*item);
                comment_closing_matches.push(None);
            }
        }
    }

    let mut final_file:Vec<u8> = Vec::with_capacity(temp_file.len());
    last_pos = 0;
    comment_opening_matches.dedup();
    comment_closing_matches.dedup();
    comment_opening_matches.sort();
    comment_closing_matches.sort();

    for (&open_pos, &close_pos) in comment_opening_matches.iter().zip(comment_closing_matches.iter()) {
        if let Some(pos) = close_pos {
            final_file.extend_from_slice(&temp_file[last_pos..open_pos]);
            last_pos = open_pos + pos + 1;
        } else {
            final_file.extend_from_slice(&temp_file[last_pos..open_pos]);
            last_pos = temp_file.len();
            break;
        }
    }
    
    if last_pos < temp_file.len() {
        final_file.extend_from_slice(&temp_file[last_pos..]);
    }
    
    final_file.into_boxed_slice()
}