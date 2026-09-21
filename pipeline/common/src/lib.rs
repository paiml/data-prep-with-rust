pub mod check;
pub mod drift;
pub mod io;
pub mod quality;
pub mod schema;
pub mod summary;

pub fn delimiter_byte(delimiter: char) -> anyhow::Result<u8> {
    if !delimiter.is_ascii() {
        anyhow::bail!("delimiter must be a single ASCII character");
    }
    Ok(delimiter as u8)
}
