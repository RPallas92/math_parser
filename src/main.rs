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

fn read_input_file() -> Result<String> {
    let input_path = "data/input.txt";
    fs::read_to_string(input_path)
}

fn eval(input: &str) -> u32 {
    let mut tokens = tokenize(input).peekable();
    parse_expression(&mut tokens)
}

fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    input.split_whitespace().map(|s| match s {
        "+" => Token::Plus,
        "-" => Token::Minus,
        "(" => Token::OpeningParenthesis,
        ")" => Token::ClosingParenthesis,
        n => Token::Operand(n.parse().unwrap()),
    })
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
            match tokens.next() {
                Some(Token::ClosingParenthesis) => val,
                other => panic!("Expected ')', got {:?}", other),
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_addition() {
        let input = "4 + 5 + 2 - 1";
        assert_eq!(eval(input), 10);
    }
}

// TODO RIcardo max performance and less memory usage
