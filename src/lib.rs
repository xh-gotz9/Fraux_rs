use std::rc::Rc;
use std::str::Chars;
use std::{collections::BTreeMap, iter::Peekable};

#[derive(Eq, PartialEq, Debug)]
pub enum BData {
    BString(String),
    Number(i32),
    List(Rc<Vec<BData>>),
    Dict(Rc<BTreeMap<String, BData>>),
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
    if let Some('i') = cv {
        let mut symb = false;
        let mut num = String::from("");
        loop {
            let v = s.next();
            match v {
                Some('0'..='9') => {
                    num.push(v.unwrap());
                }
                Some('+') | Some('-') => {
                    if symb {
                        return Err("格式错误".to_string());
                    } else {
                        num.push(v.unwrap());
                        symb = true;
                    }
                }
                Some('e') => break,
                Some(_) | None => {
                    return Err("解析错误".to_string());
                }
            }
        }
        let v: i32 = num.parse::<i32>().expect("格式错误");

        Ok(BData::Number(v))
    } else {
        return Err("解析错误".to_string());
    }
}

fn parse_string(s: &mut Peekable<Chars<'_>>) -> Result<BData, String> {
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

fn parse_dict(s: &mut Peekable<Chars>) -> Result<BData, String> {
    let p = s.next();
    if let Some('d') = p {
        let mut map = BTreeMap::new();
        loop {
            let p = s.peek();

            match p {
                Some('e') => {
                    s.next();
                    return Ok(BData::Dict(Rc::new(map)));
                }
                Some(_) => {
                    let key;
                    if let Ok(BData::BString(k)) = parse_string(s) {
                        key = k;
                    } else {
                        return Err("格式错误".to_string());
                    }

                    let v = parse_data(s);
                    match v {
                        Ok(data) => {
                            map.insert(key, data);
                        }
                        Err(_) => return v,
                    }
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

pub fn stringify(data: &BData) -> Result<String, &str> {
    let res = match data {
        BData::BString(s) => stringify_string(s),
        BData::Number(num) => stringify_number(num.clone()),
        BData::List(vec) => stringify_list(vec),
        BData::Dict(dict) => stringify_dict(dict),
    };
    res
}

fn stringify_number(data: i32) -> Result<String, &'static str> {
    let mut content = String::new();
    content.push('i');
    content.push_str(&format!("{}", data));
    content.push('e');
    Ok(content)
}

fn stringify_string(data: &str) -> Result<String, &str> {
    let mut content = String::new();
    content.push_str(&format!("{}", data.len()));
    content.push(':');
    content.push_str(data);
    Ok(content)
}

fn stringify_list(data: &Rc<Vec<BData>>) -> Result<String, &str> {
    let mut content = String::new();
    let mut err_str = "";
    content.push('l');
    if !data.iter().all(|x| match stringify(x) {
        Ok(s) => {
            content.push_str(&s);
            return true;
        }
        Err(s) => {
            err_str = s;
            return false;
        }
    }) {
        return Err(err_str);
    }

    content.push('e');
    Ok(content)
}

fn stringify_dict(data: &Rc<BTreeMap<String, BData>>) -> Result<String, &str> {
    let mut content = String::new();
    content.push('d');
    let mut err_str = "";
    if !data.iter().all(|x| {
        let mut pair = String::new();
        let key = stringify_string(x.0);
        match key {
            Ok(s) => {
                pair.push_str(&s);
            }
            Err(s) => {
                err_str = s;
                return false;
            }
        }

        let value = stringify(x.1);
        match value {
            Ok(s) => {
                pair.push_str(&s);
            }
            Err(s) => {
                err_str = s;
                return false;
            }
        };
        content.push_str(&pair);
        return true;
    }) {
        return Err(err_str);
    }

    content.push('e');
    Ok(content)
}

#[cfg(test)]
mod test {
    use super::BData;
    use std::collections::BTreeMap;
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
        parse_list_check("le", &vec![]);
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

    fn parse_dict(s: &str) -> Result<Rc<BTreeMap<String, BData>>, &str> {
        let v = super::parse(s);
        if let Ok(BData::Dict(rc)) = v {
            Ok(rc)
        } else {
            Err("err")
        }
    }

    fn parse_dict_check(s: &str, map: &BTreeMap<String, BData>) {
        let data = parse_dict(s);
        if let Ok(m) = data {
            let m = m.as_ref();
            assert_eq!(m.len(), map.len());
            m.iter().for_each(|x| {
                assert_eq!(map.contains_key(x.0), true);
                assert_eq!(x.1, map.get(x.0).unwrap());
            });
        }
    }

    #[test]
    fn parse_dict_test() {
        parse_dict_check("de", &BTreeMap::new());
        let source = "d2:k13:abce";
        let mut m = BTreeMap::new();
        m.insert("k1".to_string(), BData::BString("abc".to_string()));
        parse_dict_check(source, &m);

        let mut m = BTreeMap::new();
        let source = "d2:k13:abc2:k2l3:defi-23eee";
        m.insert("k1".to_string(), BData::BString("abc".to_string()));
        let mut k2_list = Vec::new();
        k2_list.push(BData::BString("def".to_string()));
        k2_list.push(BData::Number(-23));
        m.insert("k2".to_string(), BData::List(Rc::new(k2_list)));
        parse_dict_check(source, &m);
    }

    fn assert_stringify(s: &str, assert_s: &str) {
        if let Ok(data) = super::parse(s) {
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
        assert_stringify(s, s);
        let s = "3:lsd";
        assert_stringify(s, s);
        let s = "ld2:k12:v1ei32ee";
        assert_stringify(s, s);

        let s = "d4:key24:val24:key14:val14:key34:val34:key44:val43:key3:vale";
        let assert_s = "d3:key3:val4:key14:val14:key24:val24:key34:val34:key44:val4e";
        assert_stringify(s, assert_s);
    }
}
