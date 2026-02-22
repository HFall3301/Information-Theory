pub const ENGLISH_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub const RUSSIAN_ALPHABET: &str =
    "абвгдеёжзийклмнопрстуфхцчшщъыьэюя";
pub const ENGLISH_LEN: i32 = 26;
pub const RUSSIAN_LEN: i32 = 33;
pub fn filter_english(text: &str) -> String {
    text.chars()
        .filter(|c| {
            ENGLISH_ALPHABET.contains(c.to_ascii_lowercase())
        })
        .collect()
}
pub fn filter_russian(text: &str) -> String {
    text.chars()
        .filter(|c| {
            RUSSIAN_ALPHABET.contains(
                c.to_lowercase().next().unwrap_or(*c),
            )
        })
        .collect()
}
pub fn position(c: char, alphabet: &str) -> Option<usize> {
    alphabet.find(c)
}