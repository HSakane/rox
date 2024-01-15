use super::token::{Position, Token};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub enum ScannerError {
    Invalid(String),
}

type ScannerResult<T> = Result<T, ScannerError>;

pub struct Scanner<'a> {
    current_line: i32,
    current_column: i32,
    current_length: i32,
    chars: Peekable<Chars<'a>>,
}

impl<'a> Scanner<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self {
            current_line: 0,
            current_column: 0,
            current_length: 0,
            chars: contents.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> ScannerResult<Vec<Token>> {
        let mut tokens = vec![];
        while let Some(token) = self.next_token()? {
            match token {
                Token::WhiteSpace(_) => {}
                Token::LineFeed(_) => {}
                _ => {
                    tokens.push(token);
                }
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> ScannerResult<Option<Token>> {
        match self.chars.peek() {
            Some(c) => match c {
                c if *c == ' ' || *c == '\t' || *c == '\r' => self.skip_whitespace(),
                c if *c == '\n' => self.skip_linefeed(),
                '{' | '}' | '[' | ']' | '(' | ')' | ',' | '+' | '-' | '*' | '/' | '^' | '%'
                | '.' | ';' | '!' | '=' | '<' | '>' => self.parse_symbol(),
                '"' => {
                    self.chars.next();
                    self.current_length += 1;
                    self.parse_string_token()
                }
                c if c.is_numeric() => self.parse_number_token(),
                c if c.is_ascii_alphabetic() => self.parse_identifer_token(),
                _ => Err(ScannerError::Invalid(format!(
                    "error: an unexpected char {}, {}, {}",
                    c, self.current_line, self.current_column
                ))),
            },
            None => Ok(None),
        }
    }

    fn skip_whitespace(&mut self) -> ScannerResult<Option<Token>> {
        while let Some(c) = self.chars.peek() {
            match c {
                c if *c == ' ' || *c == '\t' || *c == '\r' => {
                    self.chars.next();
                    self.current_length += 1;
                }
                _ => break,
            }
        }
        let position = Position::new(self.current_line, self.current_column, self.current_length);
        let result = Ok(Some(Token::WhiteSpace(position)));
        self.current_column += self.current_length;
        self.current_length = 0;
        result
    }

    fn skip_linefeed(&mut self) -> ScannerResult<Option<Token>> {
        let result = Ok(Some(Token::LineFeed(Position::new(
            self.current_line,
            self.current_column,
            1,
        ))));
        self.current_line += 1;
        self.current_column = 0;
        self.current_length = 0;
        self.chars.next();
        result
    }

    fn parse_symbol(&mut self) -> ScannerResult<Option<Token>> {
        let mut length = 1;
        let mut position = Position::new(self.current_line, self.current_column, 1);
        let result = match self.chars.peek() {
            Some(c) => match c {
                '{' => Ok(Some(Token::LeftBrace(position))),
                '}' => Ok(Some(Token::RightBrace(position))),
                '[' => Ok(Some(Token::LeftBracket(position))),
                ']' => Ok(Some(Token::RightBracket(position))),
                '(' => Ok(Some(Token::LeftParen(position))),
                ')' => Ok(Some(Token::RightParen(position))),
                ',' => Ok(Some(Token::Comma(position))),
                '+' => Ok(Some(Token::Plus(position))),
                '-' => Ok(Some(Token::Minus(position))),
                '*' => Ok(Some(Token::Star(position))),
                '/' => Ok(Some(Token::Slash(position))),
                '^' => Ok(Some(Token::Pow(position))),
                '%' => Ok(Some(Token::Percent(position))),
                '.' => Ok(Some(Token::Dot(position))),
                ';' => Ok(Some(Token::Semicolon(position))),
                '!' => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some(c) => match c {
                            '=' => {
                                length = 2;
                                position.length = 2;
                                Ok(Some(Token::BangEqual(position)))
                            }
                            _ => {
                                let result = Ok(Some(Token::Bang(position)));
                                self.current_column += 1;
                                self.current_length = 0;
                                return result;
                            }
                        },
                        None => {
                            let result = Ok(Some(Token::Bang(position)));
                            self.current_column += 1;
                            self.current_length = 0;
                            return result;
                        }
                    }
                }
                '=' => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some(c) => match c {
                            '=' => {
                                length = 2;
                                position.length = 2;
                                Ok(Some(Token::EqualEqual(position)))
                            }
                            _ => {
                                let result = Ok(Some(Token::Equal(position)));
                                self.current_column += 1;
                                self.current_length = 0;
                                return result;
                            }
                        },
                        None => {
                            let result = Ok(Some(Token::Equal(position)));
                            self.current_column += 1;
                            self.current_length = 0;
                            return result;
                        }
                    }
                }
                '<' => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some(c) => match c {
                            '=' => {
                                length = 2;
                                position.length = 2;
                                Ok(Some(Token::LessEqual(position)))
                            }
                            _ => {
                                let result = Ok(Some(Token::Less(position)));
                                self.current_column += 1;
                                self.current_length = 0;
                                return result;
                            }
                        },
                        None => {
                            let result = Ok(Some(Token::Less(position)));
                            self.current_column += 1;
                            self.current_length = 0;
                            return result;
                        }
                    }
                }
                '>' => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some(c) => match c {
                            '=' => {
                                length = 2;
                                position.length = 2;
                                Ok(Some(Token::GreaterEqual(position)))
                            }
                            _ => {
                                let result = Ok(Some(Token::Greater(position)));
                                self.current_column += 1;
                                self.current_length = 0;
                                return result;
                            }
                        },
                        None => {
                            let result = Ok(Some(Token::Greater(position)));
                            self.current_column += 1;
                            self.current_length = 0;
                            return result;
                        }
                    }
                }
                _ => Err(ScannerError::Invalid(format!(
                    "error: an unexpected char {}",
                    c
                ))),
            },
            None => Ok(None),
        };
        self.current_column += length;
        self.current_length = 0;
        self.chars.next();
        result
    }

    fn parse_identifer_token(&mut self) -> ScannerResult<Option<Token>> {
        let mut ident_str = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_alphanumeric() | matches!(c, '_') {
                self.chars.next();
                self.current_length += 1;
                ident_str.push(c);
            } else {
                break;
            }
        }

        let result: ScannerResult<Option<Token>>;
        let position = Position::new(self.current_line, self.current_column, self.current_length);
        match &*ident_str {
            "and" => {
                result = Ok(Some(Token::And(position)));
            }
            "class" => {
                result = Ok(Some(Token::Class(position)));
            }
            "else" => {
                result = Ok(Some(Token::Else(position)));
            }
            "false" => {
                result = Ok(Some(Token::False(position)));
            }
            "for" => {
                result = Ok(Some(Token::For(position)));
            }
            "fun" => {
                result = Ok(Some(Token::Fun(position)));
            }
            "if" => {
                result = Ok(Some(Token::If(position)));
            }
            "null" => {
                result = Ok(Some(Token::Null(position)));
            }
            "or" => {
                result = Ok(Some(Token::Or(position)));
            }
            "return" => {
                result = Ok(Some(Token::Return(position)));
            }
            "true" => {
                result = Ok(Some(Token::True(position)));
            }
            "var" => {
                result = Ok(Some(Token::Var(position)));
            }
            "while" => {
                result = Ok(Some(Token::While(position)));
            }
            "in" => {
                result = Ok(Some(Token::In(position)));
            }
            "print" => {
                result = Ok(Some(Token::Print(position)));
            }
            "this" => {
                result = Ok(Some(Token::This(position)));
            }
            "super" => {
                result = Ok(Some(Token::Super(position)));
            }
            "to" => {
                result = Ok(Some(Token::To(position)));
            }
            _ => {
                result = Ok(Some(Token::Identifer {
                    position,
                    value: ident_str,
                }));
            }
        }
        self.current_column += self.current_length;
        self.current_length = 0;
        result
    }

    fn parse_number_token(&mut self) -> ScannerResult<Option<Token>> {
        let mut number_str = String::new();
        let mut is_float = false;
        while let Some(&c) = self.chars.peek() {
            if c.is_numeric() {
                self.chars.next();
                self.current_length += 1;
                number_str.push(c);
            } else if !is_float && matches!(c, '.') {
                is_float = true;
                self.chars.next();
                self.current_length += 1;
                number_str.push(c);
                if let Some(&c) = self.chars.peek() {
                    if !c.is_numeric() {
                        return Err(ScannerError::Invalid(format!(
                            "error: expected numeric but found '{}'.",
                            c
                        )));
                    }
                }
            } else {
                break;
            }
        }

        let position = Position::new(self.current_line, self.current_column, self.current_length);
        let result: ScannerResult<Option<Token>>;
        if is_float {
            result = match number_str.parse::<f64>() {
                Ok(number) => Ok(Some(Token::Float {
                    position,
                    value: number,
                })),
                Err(e) => Err(ScannerError::Invalid(format!("error: {}", e.to_string()))),
            };
        } else {
            result = match number_str.parse::<i64>() {
                Ok(number) => Ok(Some(Token::Integer {
                    position,
                    value: number,
                })),
                Err(e) => Err(ScannerError::Invalid(format!("error: {}", e.to_string()))),
            };
        }
        self.current_column += self.current_length;
        self.current_length = 0;
        result
    }

    fn parse_string_token(&mut self) -> ScannerResult<Option<Token>> {
        let mut utf16 = vec![];
        let mut buffer = String::new();

        while let Some(c1) = self.chars.next() {
            self.current_length += 1;
            match c1 {
                '\\' => {
                    let c2 = self.chars.next().ok_or_else(|| {
                        ScannerError::Invalid("error: a next char is expected".to_string())
                    })?;
                    self.current_length += 1;

                    if matches!(c2, '"' | '\\' | '0' | 'n' | 'r' | 't') {
                        Self::push_utf16(&mut buffer, &mut utf16)?;
                        match c2 {
                            '"' => buffer.push('"'),
                            '\\' => buffer.push('\\'),
                            '0' => buffer.push('\0'),
                            'n' => buffer.push('\n'),
                            'r' => buffer.push('\r'),
                            't' => buffer.push('\t'),
                            _ => {}
                        };
                    } else if c2 == 'u' {
                        let hexs = (0..4)
                            .filter_map(|_| {
                                let c = self.chars.next()?;
                                self.current_length += 1;
                                if c.is_ascii_hexdigit() {
                                    Some(c)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        match u16::from_str_radix(&hexs.iter().collect::<String>(), 16) {
                            Ok(code_point) => utf16.push(code_point),
                            Err(e) => {
                                return Err(ScannerError::Invalid(format!(
                                    "error: a unicode character is expected {}",
                                    e.to_string()
                                )))
                            }
                        };
                    } else {
                        return Err(ScannerError::Invalid(format!(
                            "error: an unexpected escaped char {}",
                            c2
                        )));
                    }
                }
                '"' => {
                    Self::push_utf16(&mut buffer, &mut utf16)?;
                    let position =
                        Position::new(self.current_line, self.current_column, self.current_length);
                    let result = Ok(Some(Token::String {
                        position,
                        value: buffer,
                    }));
                    self.current_column += self.current_length;
                    self.current_length = 0;
                    return result;
                }
                _ => {
                    Self::push_utf16(&mut buffer, &mut utf16)?;
                    buffer.push(c1);
                }
            }
        }
        Ok(None)
    }

    fn push_utf16(buffer: &mut String, utf16: &mut Vec<u16>) -> ScannerResult<()> {
        if utf16.is_empty() {
            return Ok(());
        }
        match String::from_utf16(utf16) {
            Ok(utf16_str) => {
                buffer.push_str(&utf16_str);
                utf16.clear();
            }
            Err(e) => {
                return Err(ScannerError::Invalid(format!("error: {}", e.to_string())));
            }
        };
        Ok(())
    }
}
