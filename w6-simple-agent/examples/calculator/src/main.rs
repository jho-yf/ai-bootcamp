use agent_core::{Agent, AgentConfig, Tool};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Minimal recursive-descent expression evaluator
//
// Supports: +  -  *  /  %  ^  parentheses, unary minus, integer and decimal
// literals.  No external dependencies required.
// ---------------------------------------------------------------------------

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Eof,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input: input.as_bytes(), pos: 0 }
    }

    fn peek(&self) -> u8 {
        if self.pos < self.input.len() { self.input[self.pos] } else { 0 }
    }

    fn advance(&mut self) -> u8 {
        let c = self.peek();
        self.pos += 1;
        c
    }

    fn skip_ws(&mut self) {
        while self.peek() == b' ' || self.peek() == b'\t' {
            self.advance();
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_ws();
        match self.peek() {
            0 => Ok(Token::Eof),
            b'+' => { self.advance(); Ok(Token::Plus) }
            b'-' => { self.advance(); Ok(Token::Minus) }
            b'*' => { self.advance(); Ok(Token::Star) }
            b'/' => { self.advance(); Ok(Token::Slash) }
            b'%' => { self.advance(); Ok(Token::Percent) }
            b'^' => { self.advance(); Ok(Token::Caret) }
            b'(' => { self.advance(); Ok(Token::LParen) }
            b')' => { self.advance(); Ok(Token::RParen) }
            c if c.is_ascii_digit() || c == b'.' => {
                let start = self.pos;
                while self.peek().is_ascii_digit() || self.peek() == b'.' {
                    self.advance();
                }
                let s = std::str::from_utf8(&self.input[start..self.pos]).unwrap();
                let n: f64 = s.parse().map_err(|_| anyhow!("Invalid number: {}", s))?;
                Ok(Token::Number(n))
            }
            c => Err(anyhow!("Unexpected character: '{}'", c as char)),
        }
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token()?;
        Ok(Self { lexer, current })
    }

    fn eat(&mut self) -> Result<Token> {
        let tok = self.current.clone();
        self.current = self.lexer.next_token()?;
        Ok(tok)
    }

    // expr = term (('+' | '-') term)*
    fn expr(&mut self) -> Result<f64> {
        let mut left = self.term()?;
        loop {
            match self.current {
                Token::Plus  => { self.eat()?; left += self.term()?; }
                Token::Minus => { self.eat()?; left -= self.term()?; }
                _ => break,
            }
        }
        Ok(left)
    }

    // term = power (('*' | '/' | '%') power)*
    fn term(&mut self) -> Result<f64> {
        let mut left = self.power()?;
        loop {
            match self.current {
                Token::Star    => { self.eat()?; left *= self.power()?; }
                Token::Slash   => {
                    self.eat()?;
                    let r = self.power()?;
                    if r == 0.0 { return Err(anyhow!("Division by zero")); }
                    left /= r;
                }
                Token::Percent => {
                    self.eat()?;
                    let r = self.power()?;
                    if r == 0.0 { return Err(anyhow!("Modulo by zero")); }
                    left %= r;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // power = unary ('^' unary)*   (right-associative)
    fn power(&mut self) -> Result<f64> {
        let base = self.unary()?;
        if self.current == Token::Caret {
            self.eat()?;
            let exp = self.power()?;
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    // unary = '-' unary | primary
    fn unary(&mut self) -> Result<f64> {
        if self.current == Token::Minus {
            self.eat()?;
            Ok(-self.unary()?)
        } else {
            self.primary()
        }
    }

    // primary = NUMBER | '(' expr ')'
    fn primary(&mut self) -> Result<f64> {
        match self.current.clone() {
            Token::Number(n) => { self.eat()?; Ok(n) }
            Token::LParen => {
                self.eat()?;
                let v = self.expr()?;
                if self.current != Token::RParen {
                    return Err(anyhow!("Expected ')'"));
                }
                self.eat()?;
                Ok(v)
            }
            ref t => Err(anyhow!("Unexpected token: {:?}", t)),
        }
    }
}

fn evaluate(expr: &str) -> Result<f64> {
    let mut parser = Parser::new(expr)?;
    let result = parser.expr()?;
    if parser.current != Token::Eof {
        return Err(anyhow!("Unexpected trailing input"));
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// calculate tool
// ---------------------------------------------------------------------------

struct CalculateTool;

#[async_trait]
impl Tool for CalculateTool {
    fn name(&self) -> &str {
        "calculate"
    }

    fn description(&self) -> &str {
        "Evaluate a mathematical expression and return the numeric result. \
         Supports +, -, *, /, %, ^ (power), parentheses, and decimal numbers."
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The mathematical expression to evaluate, e.g. '15 * 0.15'"
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let expr = args["expression"]
            .as_str()
            .ok_or_else(|| anyhow!("'expression' argument is required"))?;

        let result = evaluate(expr)?;

        // Format: avoid unnecessary trailing zeros for whole numbers.
        if result.fract() == 0.0 && result.abs() < 1e15 {
            Ok(format!("{}", result as i64))
        } else {
            Ok(format!("{}", result))
        }
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable must be set");

    let api_base = std::env::var("OPENAI_API_BASE")
        .unwrap_or_else(|_| "https://api.openai.com".to_string());

    let model = std::env::var("OPENAI_MODEL")
        .unwrap_or_else(|_| "gpt-4o".to_string());

    let mut agent = Agent::new(AgentConfig {
        model,
        system_prompt: "You are a helpful math assistant. Use the calculate tool \
                        to perform arithmetic operations accurately."
            .to_string(),
        max_steps: 20,
        api_base,
        api_key,
    });

    agent.add_tool(CalculateTool);

    let question = "What is 15% of 847, and then multiply that by 3?";
    println!("User: {}", question);
    println!();

    let reply = agent.run(question).await?;
    println!("Assistant: {}", reply);

    Ok(())
}
