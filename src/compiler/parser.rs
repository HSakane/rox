use super::{
    ast::{ExpressionNode, Program, StatementNode},
    scanner::Scanner,
    token::Token,
};

#[derive(Debug)]
pub enum ParseError {
    Invalid(String),
}

type ParseResult<T> = Result<T, ParseError>;

const PRECEDENCE_LOWEST: i32 = 0;
const PRECEDENCE_ASSIGNMENT: i32 = 5;
const PRECEDENCE_AND: i32 = 7;
const PRECEDENCE_EQUALITY: i32 = 10;
const PRECEDENCE_COMPARISON: i32 = 20;
const PRECEDENCE_TERM: i32 = 30;
const PRECEDENCE_FACTOR: i32 = 40;
const PRECEDENCE_POW: i32 = 50;
const PRECEDENCE_UNARY: i32 = 60;
const PRECEDENCE_CALL: i32 = 70;
const PRECEDENCE_PRIMARY: i32 = 80;

pub struct Parser {
    tokens: Vec<Token>,
    cur_index: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut scanner = Scanner::new(input);
        let tokens = match scanner.tokenize() {
            Ok(r) => r,
            Err(e) => panic!("{:?}", e),
        };
        Parser {
            tokens,
            cur_index: 0,
        }
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut program = Program::new();
        while let Some(_) = self.current_token() {
            let stmt = self.parse_stmt()?;
            program.stmts.push(stmt);
            self.next_token();
        }
        Ok(program)
    }

    fn parse_stmt(&mut self) -> ParseResult<StatementNode> {
        if let Some(t) = self.current_token() {
            match t {
                Token::Var(_) => self.parse_var(),
                Token::If(_) => self.parse_if(),
                Token::LeftBrace(_) => self.parse_block(),
                Token::Fun(_) => self.parse_func(),
                Token::While(_) => self.parse_while(),
                Token::Return(_) => self.parse_return(),
                Token::For(_) => self.parse_for(),
                Token::Class(_) => self.parse_class(),
                Token::Print(_) => self.parse_print(),
                _ => self.parse_expression_stmt(),
            }
        } else {
            Err(ParseError::Invalid("not statement.".to_string()))
        }
    }

    fn parse_print(&mut self) -> ParseResult<StatementNode> {
        if !matches!(self.current_token(), Some(&Token::Print(_))) {
            return Err(ParseError::Invalid(format!(
                "expected print. but found {:?}",
                self.current_token()
            )));
        }

        self.next_token();
        let expression = self.parse_expression(PRECEDENCE_LOWEST)?;

        self.next_token();
        if matches!(self.current_token(), Some(&Token::Semicolon(_))) {
            Ok(StatementNode::Print { expression })
        } else {
            Err(ParseError::Invalid(format!(
                "expected semicolon. but found {:?}",
                self.current_token()
            )))
        }
    }

    fn parse_class(&mut self) -> ParseResult<StatementNode> {
        if let Some(t) = self.current_token() {
            match t {
                Token::Class(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected class. but found {:?}",
                        self.current_token()
                    )))
                }
            }
        } else {
            return Err(ParseError::Invalid(
                "expected class. but not found.".to_string(),
            ));
        }

        self.next_token();
        let token = match self.current_token() {
            Some(t) => t,
            None => {
                return Err(ParseError::Invalid(
                    "expected identifer. but not found.".to_string(),
                ))
            }
        };
        let name = match token {
            Token::Identifer { position: _, value } => self.parse_identifer(value.clone())?,
            invalid => {
                return Err(ParseError::Invalid(format!(
                    "expected identifer. but found {:?}",
                    invalid
                )))
            }
        };

        self.next_token();
        let token = match self.current_token() {
            Some(t) => t,
            None => {
                return Err(ParseError::Invalid(
                    "expected identifer. but not found.".to_string(),
                ))
            }
        };
        let super_class = match token {
            Token::Less(_) => {
                self.next_token();
                let token = match self.current_token() {
                    Some(t) => t,
                    None => {
                        return Err(ParseError::Invalid(
                            "expected identifer. but not found.".to_string(),
                        ))
                    }
                };
                let sc = match token {
                    Token::Identifer { position: _, value } => {
                        Some(self.parse_identifer(value.clone())?)
                    }
                    invalid => {
                        return Err(ParseError::Invalid(format!(
                            "expected identifer. but found {:?}",
                            invalid
                        )))
                    }
                };
                self.next_token();
                sc
            }
            _ => None,
        };

        let body = self.parse_stmt()?;
        Ok(StatementNode::Class {
            name,
            body: Box::new(body),
            super_class,
        })
    }

    fn parse_return(&mut self) -> ParseResult<StatementNode> {
        if !matches!(self.current_token(), Some(&Token::Return(_))) {
            return Err(ParseError::Invalid(format!(
                "expected return. but found {:?}",
                self.current_token()
            )));
        }

        if matches!(self.peek_token(), Some(&Token::Semicolon(_))) {
            self.next_token();
            return Ok(StatementNode::Return { value: None });
        }

        self.next_token();
        let val = self.parse_expression(PRECEDENCE_LOWEST)?;

        self.next_token();
        if matches!(self.current_token(), Some(&Token::Semicolon(_))) {
            Ok(StatementNode::Return { value: Some(val) })
        } else {
            Err(ParseError::Invalid(format!(
                "expected semicolon. but found {:?}",
                self.current_token()
            )))
        }
    }

    fn parse_for(&mut self) -> ParseResult<StatementNode> {
        match self.current_token() {
            Some(t) => match t {
                Token::For(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected for. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected for. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        match self.current_token() {
            Some(t) => match t {
                Token::LeftParen(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected left paren. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected left paren. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let token = match self.current_token() {
            Some(t) => t,
            None => {
                return Err(ParseError::Invalid(
                    "expected any token. but not found.".to_string(),
                ))
            }
        };
        let name = match token {
            Token::Identifer { position: _, value } => self.parse_identifer(value.clone())?,
            _ => {
                return Err(ParseError::Invalid(format!(
                    "expected identifer token. but found {:?}.",
                    token
                )))
            }
        };

        self.next_token();
        match self.current_token() {
            Some(t) => match t {
                Token::In(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected in keyword. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected in keyword. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let range = self.parse_expression(PRECEDENCE_LOWEST)?;

        self.next_token();
        match self.current_token() {
            Some(t) => match t {
                Token::RightParen(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected right paren. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected right paren. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let body = self.parse_stmt()?;
        Ok(StatementNode::For {
            name,
            range,
            consequence: Box::new(body),
        })
    }

    fn parse_while(&mut self) -> Result<StatementNode, ParseError> {
        match self.current_token() {
            Some(t) => match t {
                Token::While(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected while. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected while. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        match self.current_token() {
            Some(t) => match t {
                Token::LeftParen(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected left paren. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected left paren. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let condition = self.parse_expression(PRECEDENCE_LOWEST)?;

        self.next_token();
        match self.current_token() {
            Some(t) => match t {
                Token::RightParen(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected right paren. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected right paren. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let body = self.parse_stmt()?;
        Ok(StatementNode::While {
            condition,
            consequence: Box::new(body),
        })
    }

    fn parse_func(&mut self) -> Result<StatementNode, ParseError> {
        match self.current_token() {
            Some(t) => match t {
                Token::Fun(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected fun. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected fun. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let token = match self.current_token() {
            Some(t) => t,
            None => {
                return Err(ParseError::Invalid(
                    "expected any token. but not found.".to_string(),
                ))
            }
        };
        let name = match token {
            Token::Identifer { position: _, value } => self.parse_identifer(value.clone())?,
            _ => {
                return Err(ParseError::Invalid(format!(
                    "expected identifer token. but found {:?}.",
                    token
                )))
            }
        };

        self.next_token();
        match self.current_token() {
            Some(t) => match t {
                Token::LeftParen(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected left paren. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected left paren. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let mut arguments: Vec<ExpressionNode> = Vec::new();
        while let Some(token) = self.current_token() {
            match token {
                Token::Identifer { position: _, value } => {
                    arguments.push(self.parse_identifer(value.clone())?)
                }
                Token::Comma(_) => {}
                Token::RightParen(_) => break,
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected identifer or right paren or comma. but found {:?}.",
                        token
                    )))
                }
            }
            self.next_token();
        }

        match self.current_token() {
            Some(t) => match t {
                Token::RightParen(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected right paren. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected right paren. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let body = self.parse_stmt()?;
        Ok(StatementNode::Fun {
            name,
            params: arguments,
            body: Box::new(body),
        })
    }

    fn parse_block(&mut self) -> ParseResult<StatementNode> {
        if !matches!(self.current_token(), Some(&Token::LeftBrace(_))) {
            return Err(ParseError::Invalid(format!(
                "expected left brace. but found {:?}",
                self.current_token()
            )));
        }

        self.next_token();
        let mut statements: Vec<StatementNode> = Vec::new();
        while let Some(token) = self.current_token() {
            match token {
                Token::RightBrace(_) => break,
                _ => {}
            }
            statements.push(self.parse_stmt()?);
            self.next_token();
        }

        if matches!(self.current_token(), Some(&Token::RightBrace(_))) {
            Ok(StatementNode::Block { stmts: statements })
        } else {
            Err(ParseError::Invalid(format!(
                "expected right brace. but found {:?}",
                self.current_token()
            )))
        }
    }

    fn parse_var(&mut self) -> ParseResult<StatementNode> {
        if !matches!(self.current_token(), Some(&Token::Var(_))) {
            return Err(ParseError::Invalid(format!(
                "expected var. but found {:?}",
                self.current_token()
            )));
        }

        self.next_token();
        let token = match self.current_token() {
            Some(t) => t,
            None => {
                return Err(ParseError::Invalid(
                    "expected any token. but not found.".to_string(),
                ))
            }
        };
        let name = match token {
            Token::Identifer { position: _, value } => self.parse_identifer(value.clone())?,
            _ => {
                return Err(ParseError::Invalid(format!(
                    "expected identifer token. but found {:?}.",
                    token
                )))
            }
        };

        if matches!(self.peek_token(), Some(&Token::Semicolon(_))) {
            self.next_token();
            return Ok(StatementNode::Var {
                name,
                value: ExpressionNode::NullLiteral,
            });
        }

        self.next_token();
        if !matches!(self.current_token(), Some(&Token::Equal(_))) {
            return Err(ParseError::Invalid(format!(
                "expected equal. but found {:?}",
                self.current_token()
            )));
        }

        self.next_token();
        let value = self.parse_expression(PRECEDENCE_LOWEST)?;

        self.next_token();
        if matches!(self.current_token(), Some(&Token::Semicolon(_))) {
            Ok(StatementNode::Var { name, value })
        } else {
            Err(ParseError::Invalid(format!(
                "expected semicolon. but found {:?}",
                self.current_token()
            )))
        }
    }

    fn parse_if(&mut self) -> ParseResult<StatementNode> {
        match self.current_token() {
            Some(t) => match t {
                Token::If(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected if. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected if. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        match self.current_token() {
            Some(t) => match t {
                Token::LeftParen(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected left paren. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected left paren. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let condition = self.parse_expression(PRECEDENCE_LOWEST)?;

        self.next_token();
        match self.current_token() {
            Some(t) => match t {
                Token::RightParen(_) => {}
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected right paren. but found {:?}",
                        self.current_token()
                    )))
                }
            },
            None => {
                return Err(ParseError::Invalid(
                    "expected right paren. but not found.".to_string(),
                ))
            }
        };

        self.next_token();
        let consequence = self.parse_stmt()?;

        if let Some(token) = self.peek_token() {
            match token {
                Token::Else(_) => {
                    self.next_token();
                    self.next_token();
                    let alternative = self.parse_stmt()?;
                    return Ok(StatementNode::If {
                        condition,
                        consequence: Box::new(consequence),
                        alternative: Some(Box::new(alternative)),
                    });
                }
                _ => {}
            }
        }
        Ok(StatementNode::If {
            condition,
            consequence: Box::new(consequence),
            alternative: None,
        })
    }

    fn parse_expression_stmt(&mut self) -> ParseResult<StatementNode> {
        let expression = self.parse_expression(PRECEDENCE_LOWEST)?;

        self.next_token();
        if matches!(self.current_token(), Some(&Token::Semicolon(_))) {
            Ok(StatementNode::ExpStmt { expression })
        } else {
            Err(ParseError::Invalid(format!(
                "expected semicolon. but found {:?}",
                self.current_token()
            )))
        }
    }

    fn parse_expression(&mut self, precedence: i32) -> ParseResult<ExpressionNode> {
        let token = match self.current_token() {
            Some(t) => t,
            None => {
                return Err(ParseError::Invalid(
                    "expected any token. but not found.".to_string(),
                ))
            }
        };
        let mut left = match token {
            Token::Float { position: _, value } => self.parse_float(value.clone())?,
            Token::Integer { position: _, value } => self.parse_integer(value.clone())?,
            Token::String { position: _, value } => self.parse_string(value.clone())?,
            Token::Identifer { position: _, value } => self.parse_identifer(value.clone())?,
            Token::This(_) => self.parse_identifer("this".to_string())?,
            Token::Super(_) => self.parse_identifer("super".to_string())?,
            Token::True(_) => ExpressionNode::BooleanLiteral(true),
            Token::False(_) => ExpressionNode::BooleanLiteral(false),
            Token::LeftBracket(_) => self.parse_array()?,
            Token::Null(_) => ExpressionNode::NullLiteral,
            Token::Minus(_) => self.parse_prefix("-".to_string())?,
            Token::Bang(_) => self.parse_prefix("!".to_string())?,
            Token::LeftParen(_) => self.parse_grouped()?,
            _ => {
                return Err(ParseError::Invalid(format!(
                    "expected prefix token. but found {:?}.",
                    token
                )))
            }
        };

        while let Some(token) = self.peek_token() {
            if precedence >= self.peek_precedence() {
                break;
            }

            match token {
                Token::Plus(_) => {
                    self.next_token();
                    left = self.parse_infix("+", left)?;
                }
                Token::Minus(_) => {
                    self.next_token();
                    left = self.parse_infix("-", left)?;
                }
                Token::Star(_) => {
                    self.next_token();
                    left = self.parse_infix("*", left)?;
                }
                Token::Slash(_) => {
                    self.next_token();
                    left = self.parse_infix("/", left)?;
                }
                Token::Pow(_) => {
                    self.next_token();
                    left = self.parse_infix_right("^", left)?;
                }
                Token::Percent(_) => {
                    self.next_token();
                    left = self.parse_infix_right("%", left)?;
                }
                Token::Equal(_) => {
                    match &left {
                        ExpressionNode::Identifer(_) => {}
                        ExpressionNode::Assign {
                            ope: _,
                            left: _,
                            right: _,
                        } => {}
                        ExpressionNode::IndexCall { array: _, index: _ } => {}
                        ExpressionNode::SetProperty { left: _, right: _ } => {}
                        _ => break,
                    }
                    self.next_token();
                    left = self.parse_assign("=", left)?;
                }
                Token::EqualEqual(_) => {
                    self.next_token();
                    left = self.parse_infix("==", left)?;
                }
                Token::BangEqual(_) => {
                    self.next_token();
                    left = self.parse_infix("!=", left)?;
                }
                Token::Less(_) => {
                    self.next_token();
                    left = self.parse_infix("<", left)?;
                }
                Token::LessEqual(_) => {
                    self.next_token();
                    left = self.parse_infix("<=", left)?;
                }
                Token::Greater(_) => {
                    self.next_token();
                    left = self.parse_infix(">", left)?;
                }
                Token::GreaterEqual(_) => {
                    self.next_token();
                    left = self.parse_infix(">=", left)?;
                }
                Token::And(_) => {
                    self.next_token();
                    left = self.parse_logical("and", left)?;
                }
                Token::Or(_) => {
                    self.next_token();
                    left = self.parse_logical("or", left)?;
                }
                Token::LeftParen(_) => {
                    self.next_token();
                    left = self.parse_funcall(left)?;
                }
                Token::LeftBracket(_) => {
                    self.next_token();
                    left = self.parse_indexcall(left)?;
                }
                Token::Dot(_) => {
                    self.next_token();
                    left = self.parse_property(left)?;
                }
                Token::To(_) => {
                    self.next_token();
                    left = self.parse_range(left)?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_grouped(&mut self) -> ParseResult<ExpressionNode> {
        self.next_token();
        let result = self.parse_expression(PRECEDENCE_LOWEST);
        if let Some(token) = self.peek_token() {
            match token {
                Token::RightParen(_) => self.next_token(),
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected right paren. but found {:?}.",
                        token
                    )))
                }
            }
        } else {
            return Err(ParseError::Invalid(
                "expected right paren. but not found.".to_string(),
            ));
        }
        result
    }

    fn parse_array(&mut self) -> ParseResult<ExpressionNode> {
        self.next_token();
        let precedence = self.current_precedence();
        let mut values: Vec<ExpressionNode> = Vec::new();
        while let Some(token) = self.current_token() {
            match token {
                Token::RightBracket(_) => break,
                Token::Comma(_) => {}
                _ => values.push(self.parse_expression(PRECEDENCE_LOWEST)?),
            }
            self.next_token();
        }
        Ok(ExpressionNode::ArrayLiteral(values))
    }

    fn parse_float(&mut self, value: f64) -> ParseResult<ExpressionNode> {
        Ok(ExpressionNode::FloatLiteral(value))
    }

    fn parse_integer(&mut self, value: i64) -> ParseResult<ExpressionNode> {
        Ok(ExpressionNode::IntegerLiteral(value))
    }

    fn parse_string(&mut self, value: String) -> ParseResult<ExpressionNode> {
        Ok(ExpressionNode::StringLiteral(value))
    }

    fn parse_identifer(&mut self, value: String) -> ParseResult<ExpressionNode> {
        Ok(ExpressionNode::Identifer(value))
    }

    fn parse_prefix(&mut self, ope: String) -> ParseResult<ExpressionNode> {
        self.next_token();
        let right = self.parse_expression(PRECEDENCE_UNARY)?;
        Ok(ExpressionNode::Prefix {
            ope,
            right: Box::new(right),
        })
    }

    fn parse_indexcall(&mut self, left: ExpressionNode) -> ParseResult<ExpressionNode> {
        self.next_token();
        let precedence = self.current_precedence();
        let index = self.parse_expression(PRECEDENCE_LOWEST)?;
        if let Some(token) = self.peek_token() {
            match token {
                Token::RightBracket(_) => self.next_token(),
                _ => {
                    return Err(ParseError::Invalid(format!(
                        "expected right bracket. but found {:?}.",
                        token
                    )))
                }
            }
        } else {
            return Err(ParseError::Invalid(
                "expected right bracket. but not found.".to_string(),
            ));
        }
        Ok(ExpressionNode::IndexCall {
            array: Box::new(left),
            index: Box::new(index),
        })
    }

    fn parse_funcall(&mut self, left: ExpressionNode) -> ParseResult<ExpressionNode> {
        self.next_token();
        let precedence = self.current_precedence();
        let mut parameter: Vec<ExpressionNode> = Vec::new();
        while let Some(token) = self.current_token() {
            match token {
                Token::RightParen(_) => break,
                Token::Comma(_) => {}
                _ => parameter.push(self.parse_expression(PRECEDENCE_LOWEST)?),
            }
            self.next_token();
        }
        Ok(ExpressionNode::FunCall {
            function: Box::new(left),
            arguments: parameter,
        })
    }

    fn parse_infix(
        &mut self,
        ope: impl Into<String>,
        left: ExpressionNode,
    ) -> ParseResult<ExpressionNode> {
        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        Ok(ExpressionNode::Infix {
            ope: ope.into(),
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_range(
        &mut self,
        left: ExpressionNode,
    ) -> ParseResult<ExpressionNode> {
        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        Ok(ExpressionNode::RangeLiteral {
            start: Box::new(left),
            end: Box::new(right),
        })
    }

    fn parse_property(&mut self, left: ExpressionNode) -> ParseResult<ExpressionNode> {
        let is_super = match &left {
            ExpressionNode::Identifer(name) => {
                if name == "super" {
                    true
                } else {
                    false
                }
            }
            _ => false,
        };

        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        if let Some(token) = self.peek_token() {
            if let Token::Equal(_) = token {
                return Ok(ExpressionNode::SetProperty {
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
            if let Token::LeftParen(_) = token {
                self.next_token();
                self.next_token();
                let mut parameter: Vec<ExpressionNode> = Vec::new();
                while let Some(token) = self.current_token() {
                    match token {
                        Token::RightParen(_) => break,
                        Token::Comma(_) => {}
                        _ => parameter.push(self.parse_expression(PRECEDENCE_LOWEST)?),
                    }
                    self.next_token();
                }

                if is_super {
                    return Ok(ExpressionNode::InvokeSuperMethod {
                        left: Box::new(left),
                        right: Box::new(right),
                        arguments: parameter,
                    });
                } else {
                    return Ok(ExpressionNode::InvokeMethod {
                        left: Box::new(left),
                        right: Box::new(right),
                        arguments: parameter,
                    });
                }
            }
        }
        if is_super {
            Ok(ExpressionNode::GetSuperProperty {
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Ok(ExpressionNode::GetProperty {
                left: Box::new(left),
                right: Box::new(right),
            })
        }
    }

    fn parse_infix_right(
        &mut self,
        ope: impl Into<String>,
        left: ExpressionNode,
    ) -> ParseResult<ExpressionNode> {
        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence - 1)?;
        Ok(ExpressionNode::Infix {
            ope: ope.into(),
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_assign(
        &mut self,
        ope: impl Into<String>,
        left: ExpressionNode,
    ) -> ParseResult<ExpressionNode> {
        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence - 1)?;
        Ok(ExpressionNode::Assign {
            ope: ope.into(),
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_logical(
        &mut self,
        ope: impl Into<String>,
        left: ExpressionNode,
    ) -> ParseResult<ExpressionNode> {
        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence - 1)?;
        Ok(ExpressionNode::Logical {
            ope: ope.into(),
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.cur_index)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.cur_index + 1)
    }

    fn current_precedence(&self) -> i32 {
        self.get_precedence(self.current_token())
    }

    fn peek_precedence(&self) -> i32 {
        self.get_precedence(self.peek_token())
    }

    fn get_precedence(&self, opt: Option<&Token>) -> i32 {
        match opt {
            Some(token) => match token {
                Token::Equal(_) => PRECEDENCE_ASSIGNMENT,
                Token::And(_) => PRECEDENCE_AND,
                Token::Or(_) => PRECEDENCE_AND,
                Token::EqualEqual(_) => PRECEDENCE_EQUALITY,
                Token::BangEqual(_) => PRECEDENCE_EQUALITY,
                Token::Less(_) => PRECEDENCE_COMPARISON,
                Token::LessEqual(_) => PRECEDENCE_COMPARISON,
                Token::Greater(_) => PRECEDENCE_COMPARISON,
                Token::GreaterEqual(_) => PRECEDENCE_COMPARISON,
                Token::Plus(_) => PRECEDENCE_TERM,
                Token::Minus(_) => PRECEDENCE_TERM,
                Token::Star(_) => PRECEDENCE_FACTOR,
                Token::Slash(_) => PRECEDENCE_FACTOR,
                Token::Percent(_) => PRECEDENCE_FACTOR,
                Token::Pow(_) => PRECEDENCE_POW,
                Token::LeftParen(_) => PRECEDENCE_CALL,
                Token::LeftBracket(_) => PRECEDENCE_CALL,
                Token::Dot(_) => PRECEDENCE_CALL,
                Token::To(_) => PRECEDENCE_CALL,
                _ => PRECEDENCE_LOWEST,
            },
            None => PRECEDENCE_LOWEST,
        }
    }

    fn next_token(&mut self) {
        self.cur_index += 1;
    }
}
