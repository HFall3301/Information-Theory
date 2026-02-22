use crate::alphabet::*;
use std::num::ParseIntError;
fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a.abs()
}
pub fn encrypt_decimation(
    text: &str,
    key: &str,
) -> Result<String, String> {
    let step: i32 = parse_key_number(key)?;

    if gcd(ENGLISH_LEN, step) != 1 {
        return Err(format!(
            "НОД({}, {}) должен быть равен 1.",
            ENGLISH_LEN, step
        ));
    }

    let filtered = filter_english(text);

    let result: String = filtered
        .chars()
        .map(|c| {
            let is_upper = c.is_uppercase();
            let lower = c.to_ascii_lowercase();
            let pos = position(lower, ENGLISH_ALPHABET).unwrap() as i32;

            let new_pos =
                (pos * step).rem_euclid(ENGLISH_LEN) as usize;

            let mut new_char =
                ENGLISH_ALPHABET.chars().nth(new_pos).unwrap();

            if is_upper {
                new_char = new_char.to_ascii_uppercase();
            }

            new_char
        })
        .collect();

    Ok(result)
}
pub fn decrypt_decimation(
    text: &str,
    key: &str,
) -> Result<String, String> {
    let step: i32 = parse_key_number(key)?;

    if gcd(ENGLISH_LEN, step) != 1 {
        return Err(format!(
            "НОД({}, {}) должен быть равен 1.",
            ENGLISH_LEN, step
        ));
    }

    let inverse = mod_inverse(step, ENGLISH_LEN)
        .ok_or("Не существует обратного элемента.")?;

    let filtered = filter_english(text);

    let result: String = filtered
        .chars()
        .map(|c| {
            let is_upper = c.is_uppercase();
            let lower = c.to_ascii_lowercase();
            let pos = position(lower, ENGLISH_ALPHABET).unwrap() as i32;

            let new_pos =
                (pos * inverse).rem_euclid(ENGLISH_LEN) as usize;

            let mut new_char =
                ENGLISH_ALPHABET.chars().nth(new_pos).unwrap();

            if is_upper {
                new_char = new_char.to_ascii_uppercase();
            }

            new_char
        })
        .collect();

    Ok(result)
}
pub fn encrypt_vigenere_ru(
    text: &str,
    key: &str,
) -> Result<String, String> {
    let filtered = filter_russian(text);
    let clean_key = filter_russian(key);

    if clean_key.is_empty() {
        return Err("Ключ должен содержать русские буквы.".into());
    }

    let result: String = filtered
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let is_upper = c.is_uppercase();
            let lower = c.to_lowercase().next().unwrap();

            let text_pos =
                position(lower, RUSSIAN_ALPHABET).unwrap() as i32;

            let key_char =
                clean_key.chars().nth(i % clean_key.len()).unwrap();

            let key_pos =
                position(key_char, RUSSIAN_ALPHABET).unwrap() as i32;

            let new_pos =
                (text_pos + key_pos).rem_euclid(RUSSIAN_LEN)
                    as usize;

            let mut new_char =
                RUSSIAN_ALPHABET.chars().nth(new_pos).unwrap();

            if is_upper {
                new_char =
                    new_char.to_uppercase().next().unwrap();
            }

            new_char
        })
        .collect();

    Ok(result)
}
pub fn decrypt_vigenere_ru(
    text: &str,
    key: &str,
) -> Result<String, String> {
    let filtered = filter_russian(text);
    let clean_key = filter_russian(key);

    if clean_key.is_empty() {
        return Err("Ключ должен содержать русские буквы.".into());
    }

    let result: String = filtered
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let is_upper = c.is_uppercase();
            let lower = c.to_lowercase().next().unwrap();

            let text_pos =
                position(lower, RUSSIAN_ALPHABET).unwrap() as i32;

            let key_char =
                clean_key.chars().nth(i % clean_key.len()).unwrap();

            let key_pos =
                position(key_char, RUSSIAN_ALPHABET).unwrap() as i32;

            let new_pos =
                (text_pos - key_pos).rem_euclid(RUSSIAN_LEN)
                    as usize;

            let mut new_char =
                RUSSIAN_ALPHABET.chars().nth(new_pos).unwrap();

            if is_upper {
                new_char =
                    new_char.to_uppercase().next().unwrap();
            }

            new_char
        })
        .collect();

    Ok(result)
}
fn parse_key_number(key: &str) -> Result<i32, String> {
    key.trim()
        .parse::<i32>()
        .map_err(|_: ParseIntError| {
            "Ключ должен быть целым числом.".into()
        })
}
fn mod_inverse(a: i32, m: i32) -> Option<i32> {
    (1..m).find(|x| (a * x) % m == 1)
}