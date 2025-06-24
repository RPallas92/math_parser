use std::iter::Peekable;

// TODO Ricardo fn that given a expression , creates a bigger expression with + and - so we know the result
// then we can measure perf improvements


fn main() {
    let input = "4 + 5 + 2";
    let result = eval(input);
    println!("Result is {}", result)
}

fn eval(input: &str) -> u32 {
    let tokens = tokenize(input);
    parse_expression(&mut tokens.into_iter().peekable())
}

fn tokenize(input: &str) -> Vec<Token> {
    input.split_whitespace().map(|s| match s{
        "+" => Token::Plus,
        n => Token::Operand(n.parse().unwrap())
    }).collect()
}

fn parse_expression<'a>(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> u32{
    let mut left = parse_operand(tokens);

    while let Some(Token::Plus) = tokens.peek() {
        tokens.next(); // To consume the + sign
        let right = parse_operand(tokens);
        left = left + right;
    }

    return left;
}

fn parse_operand<'a>(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> u32 {
    match tokens.next() {
        Some(Token::Operand(n)) => n,
        other => panic!("Expected number, got {:?}", other),
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Operand(u32),
    Plus
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_addition() {
        let input = "4 + 5 + 2";
        assert_eq!(eval(input), 11);
    }
}