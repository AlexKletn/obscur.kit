use std::{
    error::Error,
    io::{self, Read, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    let handle: u64 = std::env::args()
        .nth(1)
        .ok_or("missing service handle")?
        .parse()?;
    let operation = b"get";
    let mut call = vec![1];
    call.extend_from_slice(&handle.to_be_bytes());
    call.extend_from_slice(&(operation.len() as u16).to_be_bytes());
    call.extend_from_slice(operation);
    call.extend_from_slice(b"null");
    let mut output = io::stdout().lock();
    output.write_all(&(call.len() as u32).to_be_bytes())?;
    output.write_all(&call)?;
    output.flush()?;
    drop(output);

    let mut input = io::stdin().lock();
    let mut header = [0; 4];
    input.read_exact(&mut header)?;
    let size = u32::from_be_bytes(header) as usize;
    if !(2..=65536).contains(&size) {
        return Err("invalid service reply length".into());
    }
    let mut reply = vec![0; size];
    input.read_exact(&mut reply)?;
    if reply[..2] != [1, 0] {
        return Err("host rejected the service call".into());
    }
    eprintln!("{}", std::str::from_utf8(&reply[2..])?);
    Ok(())
}
