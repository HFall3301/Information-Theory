use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct LFSR {
    pub register: u64,
    pub mask: u64,
    pub size: i32,
}

impl Default for LFSR {
    fn default() -> LFSR {
        LFSR {
            register: 0b0011_1111_1111_1111_1111_1111_1111_1111,
            mask: 0b0010_0000_0000_0000_1100_0000_0000_0001, // x^30 + x^16 + x^15 + x + 1
            size: 30,
        }
    }
}

pub trait Cipher {
    fn next(&mut self) -> i32;
    fn next_byte(&mut self) -> u8 {
        let mut byte = 0u8;
        for _ in 0..8 {
            byte = (byte << 1) | (self.next() as u8);
        }
        byte
    }
}

impl Cipher for LFSR {
    fn next(&mut self) -> i32 {
        let reg = self.register;
        let output_bit = ((reg >> (self.size - 1)) & 1) as i32;
        let mut feedback = 0;

        for i in 0..self.size {
            if (self.mask >> i) & 1 == 1 {
                feedback ^= (reg >> i) & 1;
            }
        }

        let shifted = (reg & (u64::MAX >> 1)) << 1;
        let new_reg = (shifted | feedback) & ((1u64 << self.size) - 1);
        self.register = new_reg;

        output_bit
    }
}

impl Clone for LFSR {
    fn clone(&self) -> Self {
        LFSR {
            register: self.register,
            mask: self.mask,
            size: self.size,
        }
    }
}
fn create_hidden_file(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x02);
    }

    options.open(path)
}

pub fn process_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    hidden_key_path: P,
    lfsr: &mut LFSR,
    progress: Arc<AtomicU64>,
) -> std::io::Result<()> {
    let mut input = BufReader::new(File::open(input_path)?);
    let mut output = BufWriter::new(File::create(output_path)?);
    let mut key_file = BufWriter::new(create_hidden_file(hidden_key_path.as_ref())?);

    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = input.read(&mut buffer)?;
        if bytes_read == 0 { break; }

        let mut key_buffer = vec![0u8; bytes_read];
        for i in 0..bytes_read {
            key_buffer[i] = lfsr.next_byte();
            buffer[i] ^= key_buffer[i];
        }

        output.write_all(&buffer[..bytes_read])?;
        key_file.write_all(&key_buffer)?;

        progress.fetch_add(bytes_read as u64, Ordering::SeqCst);
    }

    Ok(())
}