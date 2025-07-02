use std::fs;
use std::io::Result;
use std::{iter::Peekable, time::Instant};

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

        self.pos += 1;

        let token = match byte {
            b'+' => Some(Token::Plus),
            b'-' => Some(Token::Minus),
            b'(' => Some(Token::OpeningParenthesis),
            b')' => Some(Token::ClosingParenthesis),
            b'0'..=b'9' => {
                let mut value = (byte - b'0') as u32; // TODO Ricardo SIMD improvement??
                while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                    value = 10 * value + (self.input[self.pos] - b'0') as u32;
                    self.pos += 1;
                }

                Some(Token::Operand(value))
            }
            other => panic!("Unexpected byte: '{}'", other as char),
        };

        self.pos += 1; // skip whitespace

        return token;
    }
}

fn eval(input: &[u8]) -> u32 {
    let tokenizer = Tokenizer {
        input: input,
        pos: 0,
    };
    let mut tokens = tokenizer.peekable();
    parse_expression(&mut tokens)
}

fn parse_expression(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> u32 {
    let mut left = parse_primary(tokens);

    while let Some(Token::Plus) | Some(Token::Minus) = tokens.peek() {
        let operator: Option<Token> = tokens.next();
        let right = parse_primary(tokens);
        left = match operator {
            Some(Token::Plus) => left + right,
            Some(Token::Minus) => left - right,
            other => panic!("Expected operator, got {:?}", other),
        }
    }

    return left;
}

fn parse_primary(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> u32 {
    match tokens.peek() {
        Some(Token::OpeningParenthesis) => {
            tokens.next(); // consume '('
            let val = parse_expression(tokens);
            tokens.next(); // consume ')'
            return val;
        }
        _ => parse_operand(tokens),
    }
}

fn parse_operand(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> u32 {
    match tokens.next() {
        Some(Token::Operand(n)) => n,
        other => panic!("Expected number, got {:?}", other),
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Operand(u32),
    Plus,
    Minus,
    OpeningParenthesis,
    ClosingParenthesis,
}
