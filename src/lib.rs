use std::rc::Rc;
use std::str::Chars;
use std::{collections::HashMap, iter::Peekable};

pub enum BData {
    BString(String),
    Number(i32),
    List(Rc<Vec<BData>>),
    Dict(Rc<HashMap<String, BData>>),
}

#[derive(Debug)]
pub struct ParseErr {
    message: String,
}

pub fn parse(s: &str) -> Result<BData, ParseErr> {
    let mut peekable = s.chars().peekable();
    let c = peekable.peek().unwrap();
    let res = match c {
        '0'..='9' => parse_string(&mut peekable),
        'i' => parse_number(&mut peekable),
        'l' => parse_list(&mut peekable),
        'd' => parse_dict(&mut peekable),
        _ => Err(ParseErr {
            message: String::from("非法字符"),
        }),
    };

    if peekable.peek().is_some() {
        let after: String = peekable.collect();
        println!("WARNING: unused data: {}", after);
    }
    res
}

fn parse_number(s: &mut Peekable<Chars<'_>>) -> Result<BData, ParseErr> {
    if s.next().expect("没有数据") != 'i' {
        return Err(ParseErr {
            message: "解析错误".to_string(),
        });
    }
    let mut symb = false;
    let mut num = String::from("");
    loop {
        let v = s.next().expect("没有数据");
        match v {
            '0'..='9' => {
                num.push(v);
            }
            '+' | '-' => {
                if symb {
                    return Err(ParseErr {
                        message: "格式错误".to_string(),
                    });
                } else {
                    num.push(v);
                    symb = true;
                }
            }
            'e' => break,
            _ => {
                return Err(ParseErr {
                    message: "解析错误".to_string(),
                });
            }
        }
    }
    let v: i32 = num.parse::<i32>().expect("格式错误");

    Ok(BData::Number(v))
}

pub fn parse_string(s: &mut Peekable<Chars<'_>>) -> Result<BData, ParseErr> {
    let mut len = 0;
    loop {
        let c = s.next().unwrap();
        match c {
            '0'..='9' => {
                len = len * 10 + (c as u8 - b'0');
            }
            ':' => {
                break;
            }
            _ => {
                return Err(ParseErr {
                    message: "格式错误".to_string(),
                })
            }
        }
    }

    let mut i = 0;
    let mut bstr = String::from("");

    while i < len {
        let c = s.next().unwrap();
        bstr.push(c);
        i += 1;
    }

    Ok(BData::BString(bstr))
}

fn parse_list(_s: &mut Peekable<Chars<'_>>) -> Result<BData, ParseErr> {
    Ok(BData::List(Rc::new(vec![])))
}

fn parse_dict(_s: &Peekable<Chars>) -> Result<BData, ParseErr> {
    Ok(BData::Dict(Rc::new(HashMap::new())))
}

#[cfg(test)]
mod test {
    fn parse_char(s: &str) -> String {
        let v = super::parse(s).unwrap_or(super::BData::BString("".to_string()));
        if let super::BData::BString(data) = v {
            return data;
        } else {
            return "".to_string();
        }
    }
    #[test]
    fn parse_string_test() {
        assert_eq!(parse_char("3:abc"), "abc");
        assert_eq!(parse_char("3:abcd"), "abc");
        assert_eq!(parse_char("0:"), "");
    }

    #[test]
    fn parse_number_test() {
        let num = String::from("i322e");
        let obj = super::parse(&num).expect("parse err");
        if let super::BData::Number(v) = obj {
            assert_eq!(v, 322);
            return;
        }
        panic!("parse err");
    }
}
