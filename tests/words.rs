use tijdje::words::{next_word_end, previous_word_start};

fn chars(text: &str) -> Vec<char> {
    text.chars().collect()
}

#[test]
fn previous_word_start_skips_spaces_then_the_word() {
    let text = chars("one two  three");
    assert_eq!(previous_word_start(&text, 14), 9);
    assert_eq!(previous_word_start(&text, 9), 4, "spaces before the cursor");
    assert_eq!(previous_word_start(&text, 6), 4, "inside a word");
    assert_eq!(previous_word_start(&text, 4), 0);
    assert_eq!(previous_word_start(&text, 0), 0);
}

#[test]
fn next_word_end_skips_spaces_then_the_word() {
    let text = chars("one two  three");
    assert_eq!(next_word_end(&text, 0), 3);
    assert_eq!(next_word_end(&text, 3), 7, "spaces after the cursor");
    assert_eq!(next_word_end(&text, 5), 7, "inside a word");
    assert_eq!(next_word_end(&text, 7), 14);
    assert_eq!(next_word_end(&text, 14), 14);
}

#[test]
fn a_word_is_anything_between_spaces() {
    let text = chars("a-b,c d.e");
    assert_eq!(previous_word_start(&text, 9), 6);
    assert_eq!(next_word_end(&text, 0), 5);
}

#[test]
fn only_spaces_make_no_word() {
    let text = chars("   ");
    assert_eq!(previous_word_start(&text, 3), 0);
    assert_eq!(next_word_end(&text, 0), 3);
}
