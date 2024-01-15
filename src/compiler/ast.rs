use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum StatementNode {
    Class {
        name: ExpressionNode,
        body: Box<StatementNode>,
        super_class: Option<ExpressionNode>,
    },
    For {
        name: ExpressionNode,
        range: ExpressionNode,
        consequence: Box<StatementNode>,
    },
    Fun {
        name: ExpressionNode,
        params: Vec<ExpressionNode>,
        body: Box<StatementNode>,
    },
    If {
        condition: ExpressionNode,
        consequence: Box<StatementNode>,
        alternative: Option<Box<StatementNode>>,
    },
    Return {
        value: Option<ExpressionNode>,
    },
    Var {
        name: ExpressionNode,
        value: ExpressionNode,
    },
    While {
        condition: ExpressionNode,
        consequence: Box<StatementNode>,
    },
    Block {
        stmts: Vec<StatementNode>,
    },
    Print {
        expression: ExpressionNode,
    },
    ExpStmt {
        expression: ExpressionNode,
    },
}

#[derive(Debug, Clone)]
pub enum ExpressionNode {
    Identifer(String),
    StringLiteral(String),
    FloatLiteral(f64),
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    ArrayLiteral(Vec<ExpressionNode>),
    RangeLiteral {
        start: Box<ExpressionNode>,
        end: Box<ExpressionNode>,
    },
    NullLiteral,
    Prefix {
        ope: String,
        right: Box<ExpressionNode>,
    },
    Infix {
        ope: String,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    },
    GetProperty {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    },
    GetSuperProperty {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    },
    SetProperty {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    },
    InvokeMethod {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
        arguments: Vec<ExpressionNode>,
    },
    InvokeSuperMethod {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
        arguments: Vec<ExpressionNode>,
    },
    Assign {
        ope: String,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    },
    Logical {
        ope: String,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    },
    FunCall {
        function: Box<ExpressionNode>,
        arguments: Vec<ExpressionNode>,
    },
    IndexCall {
        array: Box<ExpressionNode>,
        index: Box<ExpressionNode>,
    },
}

impl Display for StatementNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatementNode::Class {
                name,
                body,
                super_class,
            } => match &super_class {
                Some(sc) => write!(f, "class {} < {} {}", name, sc, body),
                None => write!(f, "class {} {}", name, body),
            },
            StatementNode::For {
                name,
                range,
                consequence,
            } => write!(f, "for({} in {})\r\n{}", name, range, consequence),
            StatementNode::Fun { name, params, body } => write!(
                f,
                "func {}({}){}",
                name,
                params
                    .iter()
                    .map(|exp| format!("{}", exp))
                    .collect::<Vec<_>>()
                    .join("\r\n"),
                body
            ),
            StatementNode::If {
                condition: condtion,
                consequence,
                alternative: alternatives,
            } => match alternatives {
                Some(alternatives) => write!(
                    f,
                    "if({})\r\n{}\r\nelse\r\n{}",
                    condtion, consequence, alternatives
                ),
                None => write!(f, "if({})\r\n{}", condtion, consequence),
            },
            StatementNode::Return { value } => match value {
                Some(value) => write!(f, "return {};", value),
                None => write!(f, "return;"),
            },
            StatementNode::Var { name, value } => write!(f, "var {} = {};", name, value),
            StatementNode::While {
                condition: condtion,
                consequence,
            } => write!(f, "while({})\r\n{}", condtion, consequence),
            StatementNode::Block { stmts } => write!(
                f,
                "{{\r\n{}\r\n}}",
                stmts
                    .iter()
                    .map(|exp| format!("{}", exp))
                    .collect::<Vec<_>>()
                    .join("\r\n")
            ),
            StatementNode::Print { expression } => write!(f, "print {};", expression),
            StatementNode::ExpStmt { expression } => write!(f, "{};", expression),
        }
    }
}

impl Display for ExpressionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionNode::Identifer(value) => write!(f, "{}", value),
            ExpressionNode::StringLiteral(value) => write!(f, "{}", value),
            ExpressionNode::FloatLiteral(value) => write!(f, "{}", value),
            ExpressionNode::IntegerLiteral(value) => write!(f, "{}", value),
            ExpressionNode::BooleanLiteral(value) => write!(f, "{}", value),
            ExpressionNode::ArrayLiteral(value) => write!(
                f,
                "[{}]",
                value
                    .iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ExpressionNode::RangeLiteral { start, end } => write!(f, "({}..{})", start, end),
            ExpressionNode::NullLiteral => write!(f, "null"),
            ExpressionNode::Prefix { ope, right } => write!(f, "({} {})", ope, right),
            ExpressionNode::Infix { ope, left, right } => write!(f, "({} {} {})", left, ope, right),
            ExpressionNode::GetProperty { left, right } => {
                write!(f, "({}.{})", left, right)
            }
            ExpressionNode::GetSuperProperty { left, right } => {
                write!(f, "({}.{})", left, right)
            }
            ExpressionNode::SetProperty { left, right } => {
                write!(f, "({}.{})", left, right)
            }
            ExpressionNode::InvokeMethod {
                left,
                right,
                arguments,
            } => {
                write!(
                    f,
                    "({}.{})({})",
                    left,
                    right,
                    arguments
                        .iter()
                        .map(|exp| format!("{}", exp))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ExpressionNode::InvokeSuperMethod {
                left,
                right,
                arguments,
            } => {
                write!(
                    f,
                    "({}.{})({})",
                    left,
                    right,
                    arguments
                        .iter()
                        .map(|exp| format!("{}", exp))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ExpressionNode::Assign { ope, left, right } => {
                write!(f, "({} {} {})", left, ope, right)
            }
            ExpressionNode::FunCall {
                function,
                arguments,
            } => write!(
                f,
                "{}({})",
                function,
                arguments
                    .iter()
                    .map(|exp| format!("{}", exp))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ExpressionNode::IndexCall { array, index } => write!(f, "{}[{}]", array, index),
            ExpressionNode::Logical { ope, left, right } => {
                write!(f, "({} {} {})", left, ope, right)
            }
        }
    }
}

pub struct Program {
    pub stmts: Vec<StatementNode>,
}

impl Program {
    pub fn new() -> Self {
        Program { stmts: Vec::new() }
    }
}
