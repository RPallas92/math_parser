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
                let length = integer_len(self.input, byte_pos);

                let value = match length {
                    1 => (self.input[byte_pos] - b'0') as u32,
                    2 => {
                        ((self.input[byte_pos] - b'0') as u32 * 10)
                            + (self.input[byte_pos + 1] - b'0') as u32
                    }
                    3 => {
                        ((self.input[byte_pos] - b'0') as u32 * 100)
                            + ((self.input[byte_pos + 1] - b'0') as u32 * 10)
                            + (self.input[byte_pos + 2] - b'0') as u32
                    }
                    4 => {
                        ((self.input[byte_pos] - b'0') as u32 * 1000)
                            + ((self.input[byte_pos + 1] - b'0') as u32 * 100)
                            + ((self.input[byte_pos + 2] - b'0') as u32 * 10)
                            + (self.input[byte_pos + 3] - b'0') as u32
                    }
                    _ => unreachable!(),
                };

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

// Accepts a byte slice of var length and returns the number of digits the number starting at the first byte has (pos).
// It supports between 1 and 4 digits numbers.
fn integer_len(input: &[u8], pos: usize) -> usize {
    let s = &input[pos..];

    let chunk = if s.len() >= 4 {
        unsafe { std::ptr::read_unaligned(s.as_ptr() as *const u32) }
    } else {
        let mut bytes = [b' '; 4];
        bytes[..s.len()].copy_from_slice(s);
        u32::from_le_bytes(bytes)
    };

    let mask = chunk.wrapping_sub(0x30303030);
    let non_digits_mask = (mask | mask.wrapping_add(0x76767676)) & 0x80808080;

    let length = (non_digits_mask.trailing_zeros() / 8) as usize;

    // TODO Ricardo wjhy not putting the numbers in the back [x,x, 1, 2] so we can parse the nbumber like 1 * 10 + 2 (and the non numbers anre trnasformed to 0)

    return length; // TODO Ricardo instead of match length in the caller, can we use the tree fold with bitmasking like in the article?
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
