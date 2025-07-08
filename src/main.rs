use rayon::prelude::*;
use std::fs;
use std::io::Result;
use std::time::Instant;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const NUM_THREADS: usize = 8;

#[cfg(debug_assertions)]
use dhat;

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let _profiler = dhat::Profiler::new_heap();
    let total_start = Instant::now();

    let mut step_start = Instant::now();
    let input = read_input_file()?;
    println!("Step 1: Input file read in {:?}", step_start.elapsed());

    step_start = Instant::now();
    let result = parallel_eval(&input, NUM_THREADS);
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

fn parallel_eval(input: &[u8], num_threads: usize) -> i32 {
    if num_threads <= 1 || input.len() < 1000 {
        return eval(input);
    }

    let split_indices = unsafe { find_best_split_indices_simd(input, num_threads - 1) };

    if split_indices.is_empty() {
        return eval(input);
    }

    let mut chunks = Vec::with_capacity(NUM_THREADS);
    let mut last_idx = 0;
    for &idx in &split_indices {
        chunks.push(&input[last_idx..idx - 1]);
        last_idx = idx + 2;
    }
    chunks.push(&input[last_idx..]);

    let chunk_results: Vec<i32> = chunks.par_iter().map(|&chunk| eval(chunk)).collect();

    chunk_results.into_iter().sum()
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
                let mut value = (byte - b'0') as i32;
                while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                    value = 10 * value + (self.input[self.pos] - b'0') as i32;
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

fn eval(input: &[u8]) -> i32 {
    let mut tokenizer = Tokenizer {
        input: input,
        pos: 0,
    };
    parse_expression(&mut tokenizer)
}

fn parse_expression(tokens: &mut impl Iterator<Item = Token>) -> i32 {
    let mut left = parse_primary(tokens);

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

fn parse_primary(tokens: &mut impl Iterator<Item = Token>) -> i32 {
    match tokens.next() {
        Some(Token::OpeningParenthesis) => {
            let val = parse_expression(tokens);
            return val;
        }
        Some(Token::Operand(n)) => n,
        other => panic!("Expected number, got {:?}", other),
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
unsafe fn find_best_split_indices_simd(input: &[u8], num_splits: usize) -> Vec<usize> {
    let mut final_indices = Vec::with_capacity(num_splits);
    if num_splits == 0 {
        return final_indices;
    }

    let chunk_size = input.len() / (num_splits + 1);
    let mut target_idx = 1;
    let mut last_op_at_depth_zero = 0;
    let mut depth: i32 = 0;
    let mut i = 0;
    let len = input.len();

    let open_parens = _mm512_set1_epi8(b'(' as i8);
    let close_parens = _mm512_set1_epi8(b')' as i8);
    let pluses = _mm512_set1_epi8(b'+' as i8);

    while i + 64 <= len {
        if final_indices.len() >= num_splits {
            break;
        }
        let chunk = _mm512_loadu_si512(input.as_ptr().add(i) as *const _);
        let open_mask = _mm512_cmpeq_epi8_mask(chunk, open_parens);
        let close_mask = _mm512_cmpeq_epi8_mask(chunk, close_parens);
        let plus_mask = _mm512_cmpeq_epi8_mask(chunk, pluses);

        let mut all_interesting_mask = open_mask | close_mask | plus_mask;

        let ideal_pos = target_idx * chunk_size;

        while all_interesting_mask != 0 {
            let j = all_interesting_mask.trailing_zeros() as usize;
            let current_idx = i + j;
            if (open_mask >> j) & 1 == 1 {
                depth += 1;
            } else if (close_mask >> j) & 1 == 1 {
                depth -= 1;
            } else {
                if depth == 0 {
                    last_op_at_depth_zero = current_idx;
                    if current_idx >= ideal_pos {
                        final_indices.push(current_idx);
                        target_idx += 1;
                        if final_indices.len() >= num_splits {
                            break;
                        }
                    }
                }
            }
            all_interesting_mask &= all_interesting_mask - 1;
        }
        i += 64;
    }

    let ideal_pos = target_idx * chunk_size;

    // Scalar remainder
    while i < len && final_indices.len() < num_splits {
        let char_byte = *input.get_unchecked(i);
        if char_byte == b'(' {
            depth += 1;
        } else if char_byte == b')' {
            depth -= 1;
        } else if char_byte == b'+' {
            if depth == 0 {
                last_op_at_depth_zero = i;
                if i >= ideal_pos {
                    final_indices.push(i);
                    target_idx += 1;
                }
            }
        }
        i += 1;
    }

    // Fill any remaining splits with the last found operator
    while final_indices.len() < num_splits && last_op_at_depth_zero > 0 {
        final_indices.push(last_op_at_depth_zero);
    }

    final_indices
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Operand(i32),
    Plus,
    Minus,
    OpeningParenthesis,
    ClosingParenthesis,
}
