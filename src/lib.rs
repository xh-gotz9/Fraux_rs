use std::rc::Rc;
use std::str::Chars;
use std::{collections::HashMap, iter::Peekable};

#[derive(Eq, PartialEq, Debug)]
pub enum BData {
    BString(String),
    Number(i32),
    List(Rc<Vec<BData>>),
    Dict(Rc<HashMap<String, BData>>),
}

pub fn parse(s: &str) -> Result<BData, String> {
    let mut peekable = s.chars().peekable();
    let v = parse_data(&mut peekable);
    if let Some(_) = peekable.peek() {
        let after: String = peekable.collect();
        println!("WARNING: unused data: {}", after);
    }
    v
}

fn parse_data(mut s: &mut Peekable<Chars<'_>>) -> Result<BData, String> {
    let res = match s.peek() {
        Some('0'..='9') => parse_string(&mut s),
        Some('i') => parse_number(&mut s),
        Some('l') => parse_list(&mut s),
        Some('d') => parse_dict(&mut s),
        None | Some(_) => return Err(String::from("非法字符")),
    };

    res
}

fn parse_number(s: &mut Peekable<Chars<'_>>) -> Result<BData, String> {
    let cv = s.next();
    match cv {
        Some(c) => {
            if c != 'i' {
                return Err("解析错误".to_string());
            }
        }
        None => {
            return Err("解析错误".to_string());
        }
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
                    return Err("格式错误".to_string());
                } else {
                    num.push(v);
                    symb = true;
                }
            }
            'e' => break,
            _ => {
                return Err("解析错误".to_string());
            }
        }
    }
    let v: i32 = num.parse::<i32>().expect("格式错误");

    Ok(BData::Number(v))
}

pub fn parse_string(s: &mut Peekable<Chars<'_>>) -> Result<BData, String> {
    let mut len = 0;
    loop {
        let v = s.next();
        match v {
            Some('0'..='9') => {
                len = len * 10 + (v.unwrap() as u8 - b'0');
            }
            Some(':') => {
                break;
            }
            None | Some(_) => return Err("格式错误".to_string()),
        }
    }

    let mut i = 0;
    let mut bstr = String::from("");

    while i < len {
        if let Some(c) = s.next() {
            bstr.push(c);
            i += 1;
        } else {
            return Err("格式错误".to_string());
        }
    }

    Ok(BData::BString(bstr))
}

fn parse_list(s: &mut Peekable<Chars<'_>>) -> Result<BData, String> {
    let c = s.next();
    if let Some('l') = c {
        let mut list = std::vec::Vec::new();
        loop {
            let p = s.peek();
            match p {
                Some('e') => {
                    // 结束
                    s.next();
                    return Ok(BData::List(Rc::new(list)));
                }
                Some(_) => {
                    let v = parse_data(s);
                    match v {
                        Ok(data) => {
                            list.push(data);
                        }
                        Err(_) => {
                            return v;
                        }
                    };
                }
                None => {
                    return Err("数据错误".to_string());
                }
            }
        }
    } else {
        return Err("格式错误".to_string());
    }
}

fn parse_dict(_s: &Peekable<Chars>) -> Result<BData, String> {
    Ok(BData::Dict(Rc::new(HashMap::new())))
}

#[cfg(test)]
mod test {
    use super::BData;
    use std::rc::Rc;

    fn parse_bstring(s: &str) -> Result<String, &str> {
        let v = super::parse(s);
        if let Ok(BData::BString(data)) = v {
            Ok(data)
        } else {
            Err("err")
        }
    }

    #[test]
    fn parse_bstring_test() {
        assert_eq!(parse_bstring("3:abc"), Ok("abc".to_string()));
        assert_eq!(parse_bstring("3:ab"), Err("err"));
        assert_eq!(parse_bstring("3:abcd"), Ok("abc".to_string()));
        assert_eq!(parse_bstring("0:"), Ok("".to_string()));
        assert_eq!(parse_bstring("-1:"), Err("err"));
    }

    fn parse_num(s: &str) -> Result<i32, &str> {
        let v = super::parse(s);
        if let Ok(BData::Number(data)) = v {
            Ok(data)
        } else {
            Err("err")
        }
    }

    #[test]
    fn parse_num_test() {
        assert_eq!(parse_num("i32e"), Ok(32));
        assert_eq!(parse_num("i-32e"), Ok(-32));
        assert_eq!(parse_num("i0e"), Ok(0));
        assert_eq!(parse_num("i3.2e"), Err("err"));
    }

    fn parse_list(s: &str) -> Result<Rc<Vec<BData>>, &str> {
        let v = super::parse(s);
        if let Ok(BData::List(rc)) = v {
            Ok(rc)
        } else {
            Err("err")
        }
    }

    fn parse_list_check<'b>(s: &'static str, check: &Vec<BData>) {
        let v = parse_list(s);
        match v {
            Ok(rc) => {
                let mut ch = check.iter();
                for e in rc.iter() {
                    if let Some(data) = ch.next() {
                        assert_eq!(data, e);
                    } else {
                        assert!(false);
                    }
                }
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn parse_list_test() {
        // parse_list_check("le", &vec![]);
        parse_list_check("l3:abce", &vec![BData::BString("abc".to_string())]);
        parse_list_check(
            "l3:abc4:abcde",
            &vec![
                BData::BString("abc".to_string()),
                BData::BString("abcd".to_string()),
            ],
        );
        parse_list_check(
            "l3:abci32el2:abee",
            &vec![
                BData::BString("abc".to_string()),
                BData::Number(32),
                BData::List(Rc::new(vec![BData::BString("ab".to_string())])),
            ],
        );
    }
}
