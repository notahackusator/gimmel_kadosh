use std::fs::File;
use std::io;
use std::io::Read;
use std::iter::Iterator;
use std::path::Path;

use regex::Regex;

use crate::lexer::prelude::*;
use crate::lexer::token::Token;

pub struct Regexes {
    comment_regex: Regex,
    string_regex: Regex,
    char_regex: Regex,
    identifier_regex: Regex,
    white_space: Regex,
    operator_regex: Regex,
    divider_regex: Regex,
    decimal_regex: Regex,
    integer_regex: Regex
}

impl Default for Regexes {
    fn default() -> Self {
        let comment_regex: Regex = Regex::new(r#"^//[^\n]*\n?"#).unwrap();
        let string_regex: Regex = Regex::new(r#"^"([^"\\]|\\.)*""#).unwrap();
        let char_regex: Regex = Regex::new(r#"^('[^'\\]'|('\\[\\'ntr]'))"#).unwrap();
        let identifier_regex: Regex = Regex::new(r#"^[^0-9,;:.+\-*/&|^'"?!()\[\]{}<>= \t\n][^,;:.+\-*/&|^'"?!()\[\]{}<>= \t\n]*"#).unwrap();
        let white_space: Regex = Regex::new(r#"^[ \t\n]+"#).unwrap();
        let operator_regex: Regex = Regex::new(r#"^((\+=?)|(-=?)|(\*=?)|(/=?)|(%=?)|(&&?=?)|(\|\|?=?)|(\^=?)|(!=?)|==)"#).unwrap();
        let divider_regex: Regex = Regex::new(r#"^[,;:.+\-*/&|^'"?!()\[\]{}<>=]"#).unwrap();
        let decimal_regex: Regex = Regex::new(r#"^[0-9]+\.[0-9]*"#).unwrap();
        let integer_regex: Regex = Regex::new(r#"^(0[bB][0-1]+)|(0[oO][0-7]+)|(0[xXhH][0-9a-fA-F]+)|((0[dD])?[0-9]+)"#).unwrap();
        Self { comment_regex, string_regex, char_regex, identifier_regex,
            white_space, operator_regex, divider_regex, integer_regex, decimal_regex }
    }
}

pub fn interpret_files<P>(paths: &[P], regexes: &Regexes) -> io::Result<Vec<(String, Vec<Token>)>> where P: AsRef<Path> {
    let mut codes: Vec<(String, Vec<Token>)> = Vec::with_capacity(paths.len());
    for path in paths {
        codes.push(interpret_file(path, regexes)?);
    }
    Ok(codes)
}

pub fn interpret_file<P>(path: P, regexes: &Regexes) -> io::Result<(String, Vec<Token>)> where P: AsRef<Path> {
    let mut code: String = String::new();
    File::open(path)?.read_to_string(&mut code)?;
    let code: String = code.replace("\r", "");
    let interpretation: Vec<Token> = interpret(&code, regexes);
    Ok((code, interpretation))
}

pub fn interpret(code: &str, regexes: &Regexes) -> Vec<Token> {
    let code: String = code.replace('\r', "");
    let mut tokens: Vec<Token> = Vec::new();
    let mut line: usize = 1;
    let mut col: usize = 1;
    let mut index: usize = 0;
    while index < code.len() {
        match regexes.comment_regex.find(&code[index..]) {
            None => {}
            Some(r#match) => {
                let diff: usize = r#match.end();
                index += diff;
                line += 1;
                col = 1;
                continue;
            }
        }
        match regexes.string_regex.find(&code[index..]) {
            None => {}
            Some(r#match) => {
                let diff: usize = r#match.end();
                tokens.push(Token::new(TokenType::String(r#match.as_str().into()), line, col));
                index += diff;
                col += r#match.as_str().chars().count();
                continue;
            }
        }
        match regexes.char_regex.find(&code[index..]) {
            None => {}
            Some(r#match) => {
                let diff: usize = r#match.end();
                tokens.push(Token::new(TokenType::Char(r#match.as_str().into()), line, col));
                index += diff;
                col += r#match.as_str().chars().count();
                continue;
            }
        }
        match regexes.identifier_regex.find(&code[index..]) {
            None => {}
            Some(r#match) => {
                let diff: usize = r#match.end();
                tokens.push(Token::new(TokenType::Identifier(r#match.as_str().into()), line, col));
                index += diff;
                col += r#match.as_str().chars().count();
                continue;
            }
        }
        match regexes.white_space.find(&code[index..]) {
            None => {}
            Some(r#match) => {
                let str: &str = r#match.as_str();
                let diff: usize = str.len();
                index += diff;
                let newlines: usize = str.chars().filter(|ch| *ch == '\n').count();
                if newlines == 0 {
                    col += r#match.as_str().chars().count();
                } else {
                    col = str.len() - str.rfind('\n').unwrap();
                }
                line += newlines;
                continue;
            }
        }
        match regexes.operator_regex.find(&code[index..]) {
            None => {}
            Some(r#match) => {
                let diff: usize = r#match.end();
                tokens.push(Token::new(TokenType::Operator(r#match.as_str().into()), line, col));
                index += diff;
                col += r#match.as_str().chars().count();
                continue;
            }
        }
        match regexes.divider_regex.find(&code[index..]) {
            None => {}
            Some(r#match) => {
                let diff: usize = r#match.end();
                tokens.push(Token::new(TokenType::Divider(r#match.as_str().into()), line, col));
                index += diff;
                col += r#match.as_str().chars().count();
                continue;
            }
        }
        match regexes.decimal_regex.find(&code[index..]) {
            None => {}
            Some(r#match) => {
                let diff: usize = r#match.end();
                let str: Vec<&str> = r#match.as_str().split('.').collect();
                let int: &str = str.get(0).unwrap();
                let dec: &str = str.get(1).unwrap();
                tokens.push(Token::new(TokenType::Decimal(int.into(), dec.into()), line, col));
                index += diff;
                col += r#match.as_str().chars().count();
                continue;
            }
        }
        match regexes.integer_regex.find(&code[index..]) {
            None => {}
            Some(r#match) => {
                let diff: usize = r#match.end();
                tokens.push(Token::new(TokenType::Integer(r#match.as_str().into()), line, col));
                index += diff;
                col += r#match.as_str().chars().count();
                continue;
            }
        }
    }

    tokens
}