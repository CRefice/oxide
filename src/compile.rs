use std::convert::TryInto;
use std::fmt::{self, Display};
use std::iter::Peekable;
use std::num::TryFromIntError;
use std::rc::Rc;

use crate::scan::{self, Token, TokenType, TokenType::*};
use crate::vm::{Instruction, Value};

struct VarDecl {
    name: String,
    index: u16,
}

pub struct Compiler {
    locals: Vec<VarDecl>,
    instrs: Vec<Instruction>,
}

type ScanResult = scan::Result<Token>;

fn peek<I>(it: &mut Peekable<I>) -> Result<Option<&TokenType>>
where
    I: Iterator<Item = ScanResult>,
{
    match it.peek() {
        Some(Ok(t)) => Ok(Some(&t.ttype)),
        Some(Err(e)) => Err(Error::Scan(e.clone())),
        None => Ok(None),
    }
}

fn advance<I>(it: &mut Peekable<I>) -> Result<Token>
where
    I: Iterator<Item = ScanResult>,
{
    it.next().transpose()?.ok_or(Error::EndOfInput)
}

impl Compiler {
    pub fn new() -> Self {
        let vm_owned = VarDecl {
            name: String::new(),
            index: 0,
        };
        Compiler {
            locals: vec![vm_owned],
            instrs: Vec::new(),
        }
    }

    pub fn instructions(&mut self) -> Vec<Instruction> {
        let mut chunk = Vec::new();
        std::mem::swap(&mut chunk, &mut self.instrs);
        chunk
    }

    fn emit(&mut self, instr: Instruction) {
        self.instrs.push(instr);
    }

    fn declare_local(&mut self, name: String) -> Result<u16> {
        let index: u16 = self.locals.len().try_into()?;
        self.locals.push(VarDecl { name, index });
        Ok(index)
    }

    fn find_local(&self, name: &str) -> Option<u16> {
        self.locals
            .iter()
            .rfind(|decl| decl.name == name)
            .map(|decl| decl.index)
    }

    fn stub_jump(&mut self) -> usize {
        let idx = self.instrs.len();
        self.emit(Instruction::Temp);
        idx
    }

    fn patch_jump(
        &mut self,
        src: usize,
        dst: usize,
        f: impl FnOnce(i16) -> Instruction,
    ) -> Result<()> {
        let offset = (dst - src).try_into()?;
        self.instrs[src] = f(offset);
        Ok(())
    }

    pub fn program<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        while let Some(_) = peek(it)? {
            self.declaration(it)?;
            if peek(it)?.is_some() {
                self.emit(Instruction::Pop);
            }
        }
        Ok(())
    }

    pub fn declaration<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        match peek(it)? {
            Some(Let) => self.local(it),
            Some(Global) => self.global(it),
            _ => self.expression(it),
        }
    }

    fn expression<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.or(it)
    }

    fn or<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.and(it)?;
        loop {
            if let Some(Or) = peek(it)? {
                advance(it)?;
                let jump_idx = self.stub_jump();
                self.emit(Instruction::Pop);
                self.and(it)?;
                self.patch_jump(jump_idx, self.instrs.len() - 1, Instruction::JumpIfTrue)?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn and<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.equality(it)?;
        loop {
            if let Some(And) = peek(it)? {
                advance(it)?;
                let jump_idx = self.stub_jump();
                self.emit(Instruction::Pop);
                self.equality(it)?;
                self.patch_jump(jump_idx, self.instrs.len() - 1, Instruction::JumpIfFalse)?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn equality<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.comparison(it)?;
        loop {
            match peek(it)? {
                Some(EqualEqual) | Some(BangEqual) => {
                    let op = advance(it)?;
                    self.comparison(it)?;
                    self.emit(Instruction::Equal);
                    if let BangEqual = op.ttype {
                        self.emit(Instruction::Not);
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn comparison<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.addition(it)?;
        loop {
            match peek(it)? {
                Some(Less) | Some(GreaterEqual) => {
                    let op = advance(it)?;
                    self.addition(it)?;
                    self.emit(Instruction::Less);
                    if let GreaterEqual = op.ttype {
                        self.emit(Instruction::Not);
                    }
                }
                Some(Greater) | Some(LessEqual) => {
                    let op = advance(it)?;
                    self.addition(it)?;
                    self.emit(Instruction::Greater);
                    if let LessEqual = op.ttype {
                        self.emit(Instruction::Not);
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn addition<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.multiplication(it)?;
        loop {
            match peek(it)? {
                Some(Plus) | Some(Minus) => {
                    let op = advance(it)?;
                    self.multiplication(it)?;
                    match op.ttype {
                        Plus => self.emit(Instruction::Add),
                        Minus => self.emit(Instruction::Sub),
                        _ => unreachable!(),
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn multiplication<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.unary(it)?;
        loop {
            match peek(it)? {
                Some(Star) | Some(Slash) => {
                    let op = advance(it)?;
                    self.unary(it)?;
                    match op.ttype {
                        Star => self.emit(Instruction::Mul),
                        Slash => self.emit(Instruction::Div),
                        _ => unreachable!(),
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn unary<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        match peek(it)? {
            Some(Minus) => {
                advance(it)?;
                self.unary(it)?;
                self.emit(Instruction::Neg);
            }
            Some(Not) | Some(Bang) => {
                advance(it)?;
                self.unary(it)?;
                self.emit(Instruction::Not);
            }
            _ => self.call(it)?,
        }
        Ok(())
    }

    fn call<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        self.primary(it)?;
        while let Some(LeftParen) = peek(it)? {
            let argc = self.args(it)?;
            self.emit(Instruction::Call(argc));
        }
        Ok(())
    }

    fn primary<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        let token = peek(it)?.ok_or(Error::EndOfInput)?;
        match token {
            LeftParen => self.grouping(it),
            LeftBracket => self.block(it),
            If => self.if_expr(it),
            While => self.while_expr(it),
            Function => self.fn_expr(it),
            Identifier(_) => self.variable(it),
            Literal(_) => {
                let token = advance(it)?;
                if let Literal(x) = token.ttype {
                    self.emit(Instruction::Push(x));
                    Ok(())
                } else {
                    unreachable!()
                }
            }
            _ => {
                let expected = vec![
                    LeftParen,
                    LeftBracket,
                    If,
                    While,
                    Function,
                    Identifier(String::new()),
                    Literal(Value::Null),
                ];
                let found = advance(it)?;
                Err(Error::Mismatch { expected, found })
            }
        }
    }

    fn grouping<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip LeftParen
        self.expression(it)?;
        let found = advance(it)?;
        if let RightParen = found.ttype {
            advance(it)?;
            Ok(())
        } else {
            let expected = vec![RightParen];
            Err(Error::Mismatch { expected, found })
        }
    }

    fn block<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip LeftBracket
        let frame_start = self.locals.len();
        loop {
            self.declaration(it)?;
            if let Some(RightBracket) = peek(it)? {
                advance(it)?;
                break;
            } else {
                self.emit(Instruction::Pop);
            }
        }
        let frame_len = self.locals.len() - frame_start;
        self.emit(Instruction::PopFrame(frame_len.try_into()?));
        self.locals.drain(frame_start..);
        Ok(())
    }

    fn local<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip Let
        let found = advance(it)?;
        if let Identifier(ident) = found.ttype {
            let found = advance(it)?;
            if let Equal = found.ttype {
                self.expression(it)?;
                let idx = self.declare_local(ident)?;
                self.emit(Instruction::GetLocal(idx));
                Ok(())
            } else {
                let expected = vec![Equal];
                Err(Error::Mismatch { expected, found })
            }
        } else {
            let expected = vec![Identifier(String::new())];
            Err(Error::Mismatch { expected, found })
        }
    }

    fn global<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip Global
        let found = advance(it)?;
        if let Identifier(ident) = found.ttype {
            let found = advance(it)?;
            if let Equal = found.ttype {
                self.expression(it)?;
                self.emit(Instruction::SetGlobal(ident));
                Ok(())
            } else {
                let expected = vec![Equal];
                Err(Error::Mismatch { expected, found })
            }
        } else {
            let expected = vec![Identifier(String::new())];
            Err(Error::Mismatch { expected, found })
        }
    }

    fn variable<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        let token = advance(it)?;
        let follow = peek(it)?;
        match (token.ttype, follow) {
            (Identifier(ident), Some(Equal)) => {
                advance(it)?;
                self.expression(it)?;
                if let Some(idx) = self.find_local(&ident) {
                    self.emit(Instruction::SetLocal(idx));
                } else {
                    self.emit(Instruction::SetGlobal(ident));
                }
                Ok(())
            }
            (Identifier(ident), _) => {
                if let Some(idx) = self.find_local(&ident) {
                    self.emit(Instruction::GetLocal(idx));
                } else {
                    self.emit(Instruction::GetGlobal(ident));
                }
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn if_expr<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip If
        self.expression(it)?; // Condition
        let jump_idx = self.stub_jump();
        self.emit(Instruction::Pop);
        let token = peek(it)?.ok_or(Error::EndOfInput)?;
        match token {
            Then => {
                advance(it)?;
                self.expression(it)?;
            }
            LeftBracket => {
                self.block(it)?;
            }
            _ => {
                let expected = vec![Then, LeftBracket];
                let found = advance(it)?;
                return Err(Error::Mismatch { expected, found });
            }
        };
        let jump_else_idx = self.stub_jump();
        self.emit(Instruction::Pop);
        if let Some(Else) = peek(it)? {
            advance(it)?;
            self.expression(it)?;
        } else {
            self.emit(Instruction::Push(Value::Null));
        }
        self.patch_jump(jump_else_idx, self.instrs.len() - 1, Instruction::Jump)?;
        self.patch_jump(jump_idx, jump_else_idx, Instruction::JumpIfFalse)?;
        Ok(())
    }

    fn while_expr<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip While
        self.emit(Instruction::Push(Value::Null));
        let loop_idx = self.instrs.len();
        self.expression(it)?; // Condition
        let jump_idx = self.stub_jump();
        // Pop the condition value (If jump not taken)
        self.emit(Instruction::Pop);

        // Pop last iteration's value
        self.emit(Instruction::Pop);
        if let LeftBracket = peek(it)?.ok_or(Error::EndOfInput)? {
            self.block(it)?;
        } else {
            let expected = vec![LeftBracket];
            let found = advance(it)?;
            return Err(Error::Mismatch { expected, found });
        }
        let loop_len: i16 = (self.instrs.len() - (loop_idx - 1)).try_into()?;
        self.emit(Instruction::Jump(-loop_len));
        self.patch_jump(jump_idx, self.instrs.len() - 1, Instruction::JumpIfFalse)?;
        // Pop the condition value (If jump taken)
        self.emit(Instruction::Pop);
        Ok(())
    }

    fn fn_expr<I>(&mut self, it: &mut Peekable<I>) -> Result<()>
    where
        I: Iterator<Item = ScanResult>,
    {
        advance(it)?; // Skip Fn
        let name = if let Some(Identifier(name)) = peek(it)? {
            let name = name.to_owned();
            advance(it)?;
            Some(name)
        } else {
            None
        };

        let mut fn_compiler = Compiler::new();
        let function = fn_compiler.function(name.clone(), it)?;
        self.emit(Instruction::Push(function));
        if let Some(name) = name {
            self.emit(Instruction::SetGlobal(name));
        }
        Ok(())
    }

    fn function<I>(&mut self, name: Option<String>, it: &mut Peekable<I>) -> Result<Value>
    where
        I: Iterator<Item = ScanResult>,
    {
        let arity = self.params(it)?;

        match peek(it)? {
            Some(Arrow) => {
                advance(it)?;
                self.expression(it)?;
            }
            Some(LeftBracket) => {
                self.block(it)?;
            }
            Some(_) => {
                let expected = vec![Arrow, LeftBracket];
                let found = advance(it)?;
                return Err(Error::Mismatch { expected, found });
            }
            None => {
                return Err(Error::EndOfInput);
            }
        };
        self.emit(Instruction::PopFrame(self.locals.len().try_into()?));
        self.emit(Instruction::Ret);
        Ok(Value::Function {
            chunk: Rc::new(self.instructions()),
            arity,
            name,
        })
    }

    fn params<I>(&mut self, it: &mut Peekable<I>) -> Result<usize>
    where
        I: Iterator<Item = ScanResult>,
    {
        let mut arity = 0;
        let found = advance(it)?;
        if let LeftParen = found.ttype {
            let found = advance(it)?;
            match found.ttype {
                RightParen => Ok(arity),
                Identifier(a) => {
                    self.declare_local(a)?;
                    arity = 1;
                    while let Some(Comma) = peek(it)? {
                        advance(it)?;
                        let found = advance(it)?;
                        if let Identifier(a) = found.ttype {
                            self.declare_local(a)?;
                            arity += 1;
                        } else {
                            let expected = vec![Identifier(String::new())];
                            return Err(Error::Mismatch { expected, found });
                        }
                    }
                    let found = advance(it)?;
                    if let RightParen = found.ttype {
                        Ok(arity)
                    } else {
                        let expected = vec![RightParen, Comma];
                        Err(Error::Mismatch { expected, found })
                    }
                }
                _ => {
                    let expected = vec![Identifier(String::new()), RightParen];
                    Err(Error::Mismatch { expected, found })
                }
            }
        } else {
            let expected = vec![LeftParen];
            Err(Error::Mismatch { expected, found })
        }
    }

    fn args<I>(&mut self, it: &mut Peekable<I>) -> Result<u16>
    where
        I: Iterator<Item = ScanResult>,
    {
        let mut argc = 0;
        advance(it)?; //Skip LeftParen
        match peek(it)? {
            Some(RightParen) => {
                advance(it)?;
                Ok(argc)
            }
            _ => {
                self.expression(it)?;
                argc += 1;
                while let Some(Comma) = peek(it)? {
                    advance(it)?;
                    self.expression(it)?;
                    argc += 1;
                }
                let found = advance(it)?;
                if let RightParen = found.ttype {
                    Ok(argc)
                } else {
                    let expected = vec![RightParen, Comma];
                    Err(Error::Mismatch { expected, found })
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    EndOfInput,
    Scan(scan::Error),
    Conversion(TryFromIntError),
    Mismatch {
        expected: Vec<TokenType>,
        found: Token,
    },
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Error::Conversion(err)
    }
}

impl From<scan::Error> for Error {
    fn from(err: scan::Error) -> Self {
        Error::Scan(err)
    }
}

fn human_readable_fmt<T: Display>(slice: &[T], f: &mut fmt::Formatter) -> fmt::Result {
    match slice {
        [] => write!(f, "nothing"),
        [x] => write!(f, "'{}'", x),
        [x, y] => write!(f, "'{}' or '{}'", x, y),
        _ => {
            let (last, rest) = slice.split_last().unwrap();
            let (first, middle) = rest.split_first().unwrap();
            write!(f, "one of '{}'", first)?;
            for val in middle {
                write!(f, ", '{}'", val)?;
            }
            write!(f, " or '{}'", last)
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EndOfInput => write!(f, "Unexpected end of file"),
            Error::Scan(err) => write!(f, "{}", err),
            Error::Conversion(err) => write!(f, "Number too big to fit into VM code: {}", err),
            Error::Mismatch { expected, found } => {
                write!(f, "Mismatched token: expected ")?;
                human_readable_fmt(&expected, f)?;
                write!(f, ", found '{}'", found.ttype)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Scan(err) => Some(err),
            Error::Conversion(err) => Some(err),
            _ => None,
        }
    }
}

type Result<T> = std::result::Result<T, Error>;
