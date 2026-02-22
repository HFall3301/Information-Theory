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
    let filtered_key: String = key.chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    if filtered_key.is_empty() {
        return Err("Ключ должен содержать цифры.".into());
    }

    let step: i32 = parse_key_number(&filtered_key)?;

    if gcd(ENGLISH_LEN, step) != 1 {
        return Err(format!(
            "НОД({}, {}) должен быть равен 1.",
            ENGLISH_LEN, step
        ));
    }

    let result: String = text
        .chars()
        .map(|c| {
            if let Some(pos) = position(
                c.to_ascii_lowercase(),
                ENGLISH_ALPHABET,
            ) {
                let is_upper = c.is_uppercase();
                let new_pos =
                    (pos as i32 * step)
                        .rem_euclid(ENGLISH_LEN) as usize;

                let mut new_char =
                    ENGLISH_ALPHABET.chars().nth(new_pos).unwrap();

                if is_upper {
                    new_char = new_char.to_ascii_uppercase();
                }

                new_char
            } else {
                c
            }
        })
        .collect();

    Ok(result)
}

pub fn decrypt_decimation(
    text: &str,
    key: &str,
) -> Result<String, String> {
    let filtered_key: String = key.chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    if filtered_key.is_empty() {
        return Err("Ключ должен содержать цифры.".into());
    }

    let step: i32 = parse_key_number(&filtered_key)?;

    if gcd(ENGLISH_LEN, step) != 1 {
        return Err(format!(
            "НОД({}, {}) должен быть равен 1.",
            ENGLISH_LEN, step
        ));
    }

    let inverse =
        mod_inverse(step, ENGLISH_LEN)
            .ok_or("Нет обратного элемента.")?;

    let result: String = text
        .chars()
        .map(|c| {
            if let Some(pos) = position(
                c.to_ascii_lowercase(),
                ENGLISH_ALPHABET,
            ) {
                let is_upper = c.is_uppercase();
                let new_pos =
                    (pos as i32 * inverse)
                        .rem_euclid(ENGLISH_LEN) as usize;

                let mut new_char =
                    ENGLISH_ALPHABET.chars().nth(new_pos).unwrap();

                if is_upper {
                    new_char = new_char.to_ascii_uppercase();
                }

                new_char
            } else {
                c
            }
        })
        .collect();

    Ok(result)
}

pub fn encrypt_vigenere_ru(
    text: &str,
    key: &str,
) -> Result<String, String> {
    let clean_key: String = key
        .chars()
        .filter(|c| {
            let lower = c.to_lowercase().next().unwrap_or(*c);
            RUSSIAN_ALPHABET.contains(lower)
        })
        .collect();

    if clean_key.is_empty() {
        return Err("Ключ должен содержать русские буквы.".into());
    }

    let key_chars: Vec<char> = clean_key.chars().collect();
    let key_len = key_chars.len();
    let mut key_index = 0;

    let result: String = text
        .chars()
        .map(|c| {
            let lower_c = c.to_lowercase().next().unwrap();

            if let Some(pos) = position(lower_c, RUSSIAN_ALPHABET) {
                let is_upper = c.is_uppercase();

                let key_char = key_chars[key_index % key_len];
                let key_lower = key_char.to_lowercase().next().unwrap();

                let key_pos = position(key_lower, RUSSIAN_ALPHABET)
                    .expect("Ключ содержит только русские буквы");

                println!("Шифруем: '{}' (поз={}) с ключом '{}' (поз={}) -> new_pos={}",
                         lower_c, pos, key_lower, key_pos,
                         (pos as i32 + key_pos as i32).rem_euclid(RUSSIAN_LEN));

                let new_pos = (pos as i32 + key_pos as i32)
                    .rem_euclid(RUSSIAN_LEN) as usize;

                let mut new_char = RUSSIAN_ALPHABET
                    .chars()
                    .nth(new_pos)
                    .expect("Индекс должен быть в пределах алфавита");

                if is_upper {
                    new_char = new_char
                        .to_uppercase()
                        .next()
                        .expect("Преобразование в верхний регистр");
                }

                key_index += 1;
                new_char
            } else {
                c
            }
        })
        .collect();

    Ok(result)
}

pub fn decrypt_vigenere_ru(
    text: &str,
    key: &str,
) -> Result<String, String> {
    let clean_key: String = key
        .chars()
        .filter(|c| {
            let lower = c.to_lowercase().next().unwrap_or(*c);
            RUSSIAN_ALPHABET.contains(lower)
        })
        .collect();

    if clean_key.is_empty() {
        return Err("Ключ должен содержать русские буквы.".into());
    }

    let key_chars: Vec<char> = clean_key.chars().collect();
    let key_len = key_chars.len();
    let mut key_index = 0;

    let result: String = text
        .chars()
        .map(|c| {
            let lower_c = c.to_lowercase().next().unwrap();

            if let Some(pos) = position(lower_c, RUSSIAN_ALPHABET) {
                let is_upper = c.is_uppercase();

                let key_char = key_chars[key_index % key_len];
                let key_lower = key_char.to_lowercase().next().unwrap();

                let key_pos = position(key_lower, RUSSIAN_ALPHABET)
                    .expect("Ключ содержит только русские буквы");

                println!("Расшифровываем: '{}' (поз={}) с ключом '{}' (поз={}) -> new_pos={}",
                         lower_c, pos, key_lower, key_pos,
                         (pos as i32 - key_pos as i32).rem_euclid(RUSSIAN_LEN));

                let new_pos = (pos as i32 - key_pos as i32)
                    .rem_euclid(RUSSIAN_LEN) as usize;

                let mut new_char = RUSSIAN_ALPHABET
                    .chars()
                    .nth(new_pos)
                    .expect("Индекс должен быть в пределах алфавита");

                if is_upper {
                    new_char = new_char
                        .to_uppercase()
                        .next()
                        .expect("Преобразование в верхний регистр");
                }

                key_index += 1;
                new_char
            } else {
                c
            }
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