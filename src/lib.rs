use core::slice::Iter;
use std::error::Error;
use std::{collections::BTreeMap, iter::Peekable};

#[derive(Eq, PartialEq, Debug)]
pub enum BData {
    BString(Vec<u8>),
    Number(i32),
    List(Vec<BData>),
    Dict(BTreeMap<String, BData>),
}
#[derive(Debug)]
pub enum ParseErr {
    /// 数据格式错误
    SyntaxError,
    /// 数据缺失
    DataException,
    /// 转换中出现的异常
    ParseFailure(Box<dyn Error>),
}

pub fn parse(src: &Vec<u8>) -> Result<BData, ParseErr> {
    let mut peekable: Peekable<Iter<'_, u8>> = src.iter().peekable();
    let v = parse_data(&mut peekable);
    v
}

fn parse_data(mut s: &mut Peekable<Iter<u8>>) -> Result<BData, ParseErr> {
    let res = match s.peek() {
        Some(b'0'..=b'9') => parse_string(&mut s),
        Some(b'i') => parse_number(&mut s),
        Some(b'l') => parse_list(&mut s),
        Some(b'd') => parse_dict(&mut s),
        Some(_) => return Err(ParseErr::SyntaxError),
        None => return Err(ParseErr::DataException),
    };

    res
}

fn parse_number(s: &mut Peekable<Iter<u8>>) -> Result<BData, ParseErr> {
    let cv = s.next();
    match cv {
        Some(b'i') => {
            let mut symb = false;
            let mut num = Vec::new();
            loop {
                let v = s.next();
                match v {
                    Some(b'0'..=b'9') => {
                        num.push(v.unwrap().clone());
                    }
                    Some(b'+') | Some(b'-') => {
                        if symb {
                            return Err(ParseErr::SyntaxError);
                        } else {
                            num.push(v.unwrap().clone());
                            symb = true;
                        }
                    }
                    Some(b'e') => break,
                    Some(_) => return Err(ParseErr::SyntaxError),
                    None => return Err(ParseErr::DataException),
                }
            }
            let v = String::from_utf8(num).and_then(|s| Ok(s.parse::<i32>()));

            if let Ok(v) = v {
                match v {
                    Ok(n) => {
                        return Ok(BData::Number(n));
                    }
                    Err(e) => {
                        return Err(ParseErr::ParseFailure(Box::new(e)));
                    }
                }
            } else {
                return Err(ParseErr::ParseFailure(Box::new(v.unwrap_err())));
            }
        }
        Some(_) => return Err(ParseErr::SyntaxError),
        None => return Err(ParseErr::DataException),
    }
}

fn parse_string(s: &mut Peekable<Iter<u8>>) -> Result<BData, ParseErr> {
    let mut len: usize = 0;
    loop {
        let v = s.next();
        match v {
            Some(b'0'..=b'9') => {
                len = len * 10 + (v.unwrap() - b'0') as usize;
            }
            Some(b':') => {
                break;
            }
            Some(_) => return Err(ParseErr::SyntaxError),
            None => return Err(ParseErr::DataException),
        }
    }

    let mut i = 0;
    let mut bstr = Vec::new();

    while i < len {
        match s.next() {
            Some(c) => {
                bstr.push(c.clone());
                i += 1;
            }
            None => return Err(ParseErr::DataException),
        }
    }

    Ok(BData::BString(bstr))
}

fn parse_list(s: &mut Peekable<Iter<u8>>) -> Result<BData, ParseErr> {
    let c = s.next();
    match c {
        Some(b'l') => {
            let mut list = std::vec::Vec::new();
            loop {
                let p = s.peek();
                match p {
                    Some(b'e') => {
                        s.next();
                        return Ok(BData::List(list));
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
                        return Err(ParseErr::DataException);
                    }
                }
            }
        }
        Some(_) => return Err(ParseErr::SyntaxError),
        None => return Err(ParseErr::DataException),
    }
}

fn parse_dict(s: &mut std::iter::Peekable<std::slice::Iter<'_, u8>>) -> Result<BData, ParseErr> {
    let p = s.next();
    match p {
        Some(b'd') => {
            let mut map = BTreeMap::new();
            loop {
                let p = s.peek();

                match p {
                    Some(b'e') => {
                        s.next();
                        return Ok(BData::Dict(map));
                    }
                    Some(_) => {
                        let data = parse_string(s);
                        let key;
                        match data {
                            Ok(BData::BString(k)) => key = k,
                            Ok(_) => return Err(ParseErr::SyntaxError),
                            Err(_) => return data,
                        }

                        if let Ok(k) = String::from_utf8(key) {
                            let v = parse_data(s);
                            match v {
                                Ok(data) => {
                                    map.insert(k, data);
                                }
                                Err(_) => return v,
                            }
                        }
                    }
                    None => return Err(ParseErr::DataException),
                }
            }
        }
        Some(_) => return Err(ParseErr::SyntaxError),
        None => return Err(ParseErr::DataException),
    }
}

pub fn stringify(data: &BData) -> Result<Vec<u8>, &str> {
    let res = match data {
        BData::BString(s) => stringify_string(s),
        BData::Number(num) => stringify_number(num),
        BData::List(vec) => stringify_list(vec),
        BData::Dict(dict) => stringify_dict(dict),
    };
    res
}

fn stringify_number(data: &i32) -> Result<Vec<u8>, &'static str> {
    let mut content = Vec::new();
    content.push(b'i');
    content.append(&mut format!("{}", data).as_bytes().to_vec());
    content.push(b'e');
    Ok(content)
}

fn stringify_string(data: &Vec<u8>) -> Result<Vec<u8>, &'static str> {
    let mut content = Vec::new();
    content.append(&mut format!("{}", data.len()).as_bytes().to_vec());
    content.push(b':');
    content.append(&mut data.clone());
    Ok(content)
}

fn stringify_list(data: &Vec<BData>) -> Result<Vec<u8>, &str> {
    let mut content = Vec::new();
    let mut err_str = "";
    content.push(b'l');
    if !data.iter().all(|x| match stringify(x).as_mut() {
        Ok(s) => {
            content.append(s);
            return true;
        }
        Err(s) => {
            err_str = s;
            return false;
        }
    }) {
        return Err(err_str);
    }

    content.push(b'e');
    Ok(content)
}

fn stringify_dict(data: &BTreeMap<String, BData>) -> Result<Vec<u8>, &str> {
    let mut content = Vec::new();
    content.push(b'd');
    let mut err_str = "";
    if !data.iter().all(|x| {
        let key = stringify_string(&x.0.as_bytes().to_vec());
        match key {
            Ok(mut s) => {
                content.append(&mut s);
            }
            Err(s) => {
                err_str = s;
                return false;
            }
        }

        let value = stringify(x.1);
        match value {
            Ok(mut s) => {
                content.append(&mut s);
            }
            Err(s) => {
                err_str = s;
                return false;
            }
        };
        return true;
    }) {
        return Err(err_str);
    }

    content.push(b'e');
    Ok(content)
}

#[cfg(test)]
mod test {
    use super::BData;
    use std::collections::BTreeMap;

    fn parse_bstring(s: &str) -> Result<String, &str> {
        let v = super::parse(&s.as_bytes().to_vec());
        if let Ok(BData::BString(data)) = v {
            Ok(String::from_utf8(data).unwrap())
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
        let v = super::parse(&s.as_bytes().to_vec());
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
        assert_eq!(
            parse_num(&format!("i{}e", i64::MAX).to_string()),
            Err("err")
        );
    }

    fn parse_list(s: &str) -> Result<Vec<BData>, &str> {
        let v = super::parse(&s.as_bytes().to_vec());
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
        parse_list_check("le", &vec![]);
        parse_list_check("l3:abce", &vec![BData::BString("abc".as_bytes().to_vec())]);
        parse_list_check(
            "l3:abc4:abcde",
            &vec![
                BData::BString("abc".as_bytes().to_vec()),
                BData::BString("abcd".as_bytes().to_vec()),
            ],
        );
        parse_list_check(
            "l3:abci32el2:abee",
            &vec![
                BData::BString("abc".as_bytes().to_vec()),
                BData::Number(32),
                BData::List(vec![BData::BString("ab".as_bytes().to_vec())]),
            ],
        );
    }

    fn parse_dict(s: &str) -> Result<BTreeMap<String, BData>, &str> {
        let v = super::parse(&s.as_bytes().to_vec());
        if let Ok(BData::Dict(map)) = v {
            Ok(map)
        } else {
            Err("err")
        }
    }

    fn parse_dict_check(s: &str, map: &BTreeMap<String, BData>) {
        let data = parse_dict(s);

        let m = data.expect("parse dict failed");

        assert_eq!(m.len(), map.len());
        m.iter().for_each(|x| {
            assert_eq!(map.contains_key(x.0), true);
            assert_eq!(x.1, map.get(x.0).unwrap());
        });
    }

    #[test]
    fn parse_dict_test() {
        parse_dict_check("de", &BTreeMap::new());
        let source = "d2:k13:abce";
        let mut m = BTreeMap::new();
        m.insert("k1".to_string(), BData::BString("abc".as_bytes().to_vec()));
        parse_dict_check(source, &m);

        let mut m = BTreeMap::new();
        let source = "d2:k13:abc2:k2l3:defi-23eee";
        m.insert("k1".to_string(), BData::BString("abc".as_bytes().to_vec()));
        let mut k2_list = Vec::new();
        k2_list.push(BData::BString("def".as_bytes().to_vec()));
        k2_list.push(BData::Number(-23));
        m.insert("k2".to_string(), BData::List(k2_list));
        parse_dict_check(source, &m);
    }

    fn assert_stringify(s: &str, assert_s: Vec<u8>) {
        if let Ok(data) = super::parse(&s.as_bytes().to_vec()) {
            let stringify = super::stringify(&data);
            println!("parse: {}", s);
            if let Ok(st) = stringify {
                assert_eq!(st, assert_s);
            } else {
                panic!("stringify error!");
            }
        } else {
            panic!("parse error!");
        }
    }

    #[test]
    fn stringify_test() {
        let s = "3:abc";
        assert_stringify(s, s.as_bytes().to_vec());
        let s = "3:lsd";
        assert_stringify(s, s.as_bytes().to_vec());
        let s = "ld2:k12:v1ei32ee";
        assert_stringify(s, s.as_bytes().to_vec());

        let s = "d4:key24:val24:key14:val14:key34:val34:key44:val43:key3:vale";
        let assert_s = "d3:key3:val4:key14:val14:key24:val24:key34:val34:key44:val4e"
            .as_bytes()
            .to_vec();
        assert_stringify(s, assert_s);
    }
}
