use std::fs;
use std::io::Result;

fn main() -> Result<()> {
    let base_input = "( ( 400 + 50 ) + 2 + 3000 + 200 - 1000 )"; // 2652
    let mut input = base_input.to_string();

    for i in 0..50_000_000 {
        let infix = if i % 2 == 1 { " + " } else { " - " };
        input += infix;
        input += base_input;
    }

    let path = "data/input.txt";
    fs::write(path, input)?;

    println!("Test file created on {}", path);
    Ok(())
}
