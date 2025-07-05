use std::fs;
use std::io::Result;
use std::time::Instant;

/*#[cfg(debug_assertions)]
use dhat;

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;*/

fn main() -> Result<()> {
    /*#[cfg(debug_assertions)]
    let _profiler = dhat::Profiler::new_heap();*/
    let total_start = Instant::now();

    let mut step_start = Instant::now();
    let input = read_input_file()?;
    println!("Step 1: Input file read in {:?}", step_start.elapsed());

    step_start = Instant::now();
    let result = eval(&input);
    println!(
        "Step 2: Calculation completed in {:?}",
        step_start.elapsed()
    );

    let total_duration = total_start.elapsed();

    println!("\n--- Summary ---");
    println!("Result: {}", result);
    println!("Total time: {:?}", total_duration);

    Ok(())
}

fn read_input_file() -> Result<Vec<u8>> {
    fs::read("data/input.txt")
}

struct Tokenizer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        let byte = self.input[self.pos];
        let byte_pos = self.pos;

        self.pos += 1;

        let token = match byte {
            b'+' => Some(Token::Plus),
            b'-' => Some(Token::Minus),
            b'(' => Some(Token::OpeningParenthesis),
            b')' => Some(Token::ClosingParenthesis),
            b'0'..=b'9' => {
                let (value, length) = parse_integer(&self.input[byte_pos..]);
                self.pos += length - 1;
                Some(Token::Operand(value))
            }
            other => panic!("Unexpected byte: '{}'", other as char),
        };

        self.pos += 1; // skip whitespace

        return token;
    }
}

fn eval(input: &[u8]) -> u32 {
    let mut tokenizer = Tokenizer {
        input: input,
        pos: 0,
    };
    parse_expression(&mut tokenizer)
}

fn parse_expression(tokens: &mut impl Iterator<Item = Token>) -> u32 {
    let mut left = parse_primary(tokens);

    // TODO ricardo next optimization. do not use peekable, and do not use peek, just iterate and if the token is not operand, break. then match on the operand

    while let Some(token) = tokens.next() {
        if token == Token::ClosingParenthesis {
            break;
        }

        let right = parse_primary(tokens);
        left = match token {
            Token::Plus => left + right,
            Token::Minus => left - right,
            other => panic!("Expected operator, got {:?}", other),
        }
    }

    return left;
}

fn parse_primary(tokens: &mut impl Iterator<Item = Token>) -> u32 {
    match tokens.next() {
        Some(Token::OpeningParenthesis) => {
            let val = parse_expression(tokens);
            return val;
        }
        Some(Token::Operand(n)) => n,
        other => panic!("Expected number, got {:?}", other),
    }
}

/// Parses an integer from the start of a slice using SWAR techniques.
/// Returns the parsed value and the number of bytes consumed.
fn parse_integer(s: &[u8]) -> (u32, usize) {
    // Step 1: Read up to 4 bytes from the slice.
    let chunk = if s.len() >= 4 {
        // Fast path: read directly from the slice if there's enough room.
        unsafe { std::ptr::read_unaligned(s.as_ptr() as *const u32) }
    } else {
        // Slow path: copy to a padded buffer for the end of the file.
        let mut bytes = [b' '; 4]; // Pad with a non-digit
        bytes[..s.len()].copy_from_slice(s);
        u32::from_le_bytes(bytes)
    };

    // Step 2: Find the length of the number (1-4) using bitmasking.
    let mask = chunk.wrapping_sub(0x30303030);
    let non_digits_mask = (mask | mask.wrapping_add(0x76767676)) & 0x80808080;
    let length = (non_digits_mask.trailing_zeros() / 8) as usize;

    // Step 3: Right-align the digits and convert from ASCII to integer values.
    // This implements the "delete and shift" idea.
    let shift = (4 - length) * 8;
    // Shift the ASCII digits to the right, leaving garbage on the left.
    let aligned_chunk = chunk << shift;
    // Create a subtraction mask that is also shifted.
    let sub_mask = 0x30303030 << shift;
    // Subtract to convert from ASCII. The garbage on the left is irrelevant.
    let digits = aligned_chunk.wrapping_sub(sub_mask);

    // Step 4: Parse the number from the bytes using a parallel, tree-like approach.
    let lower_digits = digits & 0x00ff00ff;
    let upper_digits = (digits >> 8) & 0x00ff00ff;
    let temp = lower_digits + upper_digits * 10;

    let value = ((temp & 0x0000ffff) + (temp >> 16) * 100) as u32;

    (value, length)
}
// TODO RICSRDO COMPARE WITH PREVIOUS IMPL BUT USE as U32 otherwise it fails!!!!!!!!!!!!!!!!!!!!!!!!!
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// !!!!!!!!!!!!!!!!!!!

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Operand(u32),
    Plus,
    Minus,
    OpeningParenthesis,
    ClosingParenthesis,
}

// let's say we support a variable length of between 1 and 4 digts

// https://rust-malaysia.github.io/code/2020/07/11/faster-integer-parsing.html

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hack() {
        let input = "123".as_bytes();
        let result = integer_len(input, 0);
        assert_eq!(result, 3);
    }
}
