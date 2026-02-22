pub const ENGLISH_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub const RUSSIAN_ALPHABET: &str =
    "абвгдеёжзийклмнопрстуфхцчшщъыьэюя";
pub const ENGLISH_LEN: i32 = 26;
pub const RUSSIAN_LEN: i32 = 33;
pub fn position(c: char, alphabet: &str) -> Option<usize> {
    alphabet.chars().position(|x| x == c)
}