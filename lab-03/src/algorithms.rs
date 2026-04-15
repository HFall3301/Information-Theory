use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct RsaParams {
    pub p: u64,
    pub q: u64,
    pub r: u64,
    pub phi: u64,
    pub d: u64,
    pub e: u64,
}

pub fn build_rsa_params(p: u64, q: u64, d: u64) -> Result<RsaParams, String> {
    if p < 2 || !is_prime(p) {
        return Err("p должно быть простым числом (> 1).".to_string());
    }
    if q < 2 || !is_prime(q) {
        return Err("q должно быть простым числом (> 1).".to_string());
    }
    if p == q {
        return Err("p и q не должны совпадать.".to_string());
    }

    let r = p.checked_mul(q).ok_or_else(|| "Переполнение при вычислении r = p*q.".to_string())?;

    if r <= 255 {
        return Err("r = p*q должно быть больше 255 (чтобы шифровать любой байт 0..255).".to_string());
    }
    if r > u16::MAX as u64 {
        return Err("r = p*q должно помещаться в 2 байта (<= 65535).".to_string());
    }

    let phi = (p - 1)
        .checked_mul(q - 1)
        .ok_or_else(|| "Переполнение при вычислении функции Эйлера.".to_string())?;

    if d <= 1 || d >= phi {
        return Err("Закрытый ключ d должен быть: 1 < d < phi(r).".to_string());
    }

    if gcd(d, phi) != 1 {
        return Err("d и phi(r) должны быть взаимно простыми (НОД = 1).".to_string());
    }

    let e = mod_inverse(d, phi)
        .ok_or_else(|| "Не удалось найти открытый ключ e как обратный к d по mod phi(r).".to_string())?;

    Ok(RsaParams { p, q, r, phi, d, e })
}

pub fn encrypt_file<P: AsRef<Path>>(input_path: P, output_path: P, params: &RsaParams) -> io::Result<()> {
    let mut input = BufReader::new(File::open(input_path)?);
    let mut output = BufWriter::new(File::create(output_path)?);

    let mut buffer = [0u8; 8192];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        for &byte in &buffer[..read] {
            let m = byte as u64;
            if m >= params.r {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Шифруемый байт должен быть меньше модуля r.",
                ));
            }

            let c = mod_pow(m, params.e, params.r);
            let c16 = u16::try_from(c).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Зашифрованный блок не поместился в 16 бит.",
                )
            })?;

            output.write_all(&c16.to_be_bytes())?;
        }
    }

    output.flush()?;
    Ok(())
}

pub fn decrypt_file<P: AsRef<Path>>(input_path: P, output_path: P, params: &RsaParams) -> io::Result<()> {
    let mut input = BufReader::new(File::open(&input_path)?);
    let mut output = BufWriter::new(File::create(output_path)?);

    let file_len = std::fs::metadata(input_path)?.len();
    if file_len % 2 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Размер зашифрованного файла должен быть кратен 2 байтам.",
        ));
    }

    let mut pair = [0u8; 2];
    loop {
        match input.read_exact(&mut pair) {
            Ok(()) => {
                let c = u16::from_be_bytes(pair) as u64;
                if c >= params.r {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Зашифрованный блок {c} должен быть меньше r = {}.", params.r),
                    ));
                }

                let m = mod_pow(c, params.d, params.r);
                let m8 = u8::try_from(m).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "После дешифрования получено значение вне диапазона байта 0..255.",
                    )
                })?;

                output.write_all(&[m8])?;
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }

    output.flush()?;
    Ok(())
}

pub fn read_cipher_blocks<P: AsRef<Path>>(path: P) -> io::Result<Vec<u16>> {
    let data = std::fs::read(path)?;
    if data.len() % 2 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Файл шифротекста имеет нечётную длину и не может состоять из 16-битных блоков.",
        ));
    }

    let mut blocks = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(2) {
        blocks.push(u16::from_be_bytes([chunk[0], chunk[1]]));
    }
    Ok(blocks)
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n % 2 == 0 {
        return n == 2;
    }

    let mut i = 3u64;
    while i.saturating_mul(i) <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn extended_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    if b == 0 {
        return (a, 1, 0);
    }

    let (g, x1, y1) = extended_gcd(b, a % b);
    (g, y1, x1 - (a / b) * y1)
}

fn mod_inverse(a: u64, modulus: u64) -> Option<u64> {
    let (g, x, _) = extended_gcd(a as i128, modulus as i128);
    if g != 1 {
        return None;
    }

    let m = modulus as i128;
    Some((((x % m) + m) % m) as u64)
}

fn mod_pow(base: u64, exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }

    let mut result: u128 = 1;
    let mut base_acc: u128 = (base % modulus) as u128;
    let mut power = exp;
    let m = modulus as u128;

    while power > 0 {
        if power & 1 == 1 {
            result = (result * base_acc) % m;
        }
        base_acc = (base_acc * base_acc) % m;
        power >>= 1;
    }

    result as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path(name: &str) -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("lab03_rsa_{name}_{ts}.bin"))
    }

    #[test]
    fn rsa_params_are_built_correctly() {
        let params = build_rsa_params(251, 241, 17).expect("valid rsa params");
        assert_eq!(params.r, 60491);
        assert_eq!(params.phi, 60000);
        assert_eq!(gcd(params.d, params.phi), 1);
        assert_eq!((params.e * params.d) % params.phi, 1);
    }

    #[test]
    fn rsa_params_reject_non_prime_input() {
        let err = build_rsa_params(256, 241, 17).expect_err("p=256 is not prime");
        assert!(err.contains("простым"));
    }

    #[test]
    fn encrypt_then_decrypt_round_trip() {
        let params = build_rsa_params(251, 241, 17).expect("valid rsa params");
        let input_path = unique_path("in");
        let cipher_path = unique_path("cipher");
        let output_path = unique_path("out");

        let source = vec![0u8, 1, 2, 42, 127, 200, 255];
        std::fs::write(&input_path, &source).expect("write input");

        encrypt_file(&input_path, &cipher_path, &params).expect("encrypt");
        let blocks = read_cipher_blocks(&cipher_path).expect("read blocks");
        assert_eq!(blocks.len(), source.len());

        decrypt_file(&cipher_path, &output_path, &params).expect("decrypt");
        let restored = std::fs::read(&output_path).expect("read output");
        assert_eq!(restored, source);

        let _ = std::fs::remove_file(input_path);
        let _ = std::fs::remove_file(cipher_path);
        let _ = std::fs::remove_file(output_path);
    }
}

