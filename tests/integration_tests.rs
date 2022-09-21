#![allow(dead_code)]    
#[cfg(feature="gc3c")]
extern crate gc3c;
#[macro_use]
extern crate pratt;

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

#[cfg(feature="gc3c")]
use gc3c::{InGcEnv, gc, Mark};

use pratt::{PrattBox, Token, Symbol, Pratt, Tokenizer};
use pratt::dyn::{DynamicToken, DynamicSymbol };
    
    

type FnewToken<S> = Box<dyn Fn(&str) -> PrattBox<S>>;
   
#[derive(Debug, PartialEq)]
enum TokenStatus {
    Init,
    InToken,
    InQuote,
    InString,
    InNum,
    EndToken,
}
 
struct TokenizerStatus {
    status: TokenStatus,
    i: usize,
    j: usize,
    inquotepar: u32,
}

struct StringTokenizer<S: Symbol> {
    input: String,
    tokens : RefCell<Vec<PrattBox<S>>>,
    map: RefCell<HashMap<&'static str, FnewToken<S>>>,
    st: RefCell<TokenizerStatus>,
}

impl<S: Symbol> StringTokenizer<S> {
    fn new(input: String) -> StringTokenizer<S> {
        StringTokenizer {
            input: input,
            st: RefCell::new(
                TokenizerStatus {
                          status: TokenStatus::Init, 
                          i: 0, j: 0,
                          inquotepar: 0,
                }
            ),
            tokens: RefCell::new(Vec::new()),
            map: RefCell::new(HashMap::new()),
        }
    }
    fn new_token(&self, typ: &str, s: &str) -> PrattBox<S> {
        if let Some(ref f ) = self.map.borrow().get(typ) {
            f(s)
        } else {
            unreachable!("cannot handle token");
        }
    }

    fn register_token(& self, s: &'static str, f: FnewToken<S>) {
        self.map.borrow_mut().insert(s, f);
    }
}

impl<S: Symbol> Tokenizer<S> for StringTokenizer<S> {

    fn current(&self) -> Option<PrattBox<S>> {
        let tokens = self.tokens.borrow();
        tokens.last().map(|e| { e.clone() }) 
    }

    fn advance(& self)  {
        let mut st = self.st.borrow_mut();
        while st.j < self.input.len() && st.i < self.input.len() {
            match st.status { 
                TokenStatus::Init => {
                    println!("1: {:?}", &self.input[st.j..st.j+1]);
                    match &self.input[st.j..st.j+1] {
                        " " | "\t" | "\n"  => {
                            st.j = st.j + 1;
                            st.i = st.j;
                        }
                        "'" => {
                            st.j = st.j + 1;
                            st.i = st.j;
                            st.status = TokenStatus::InQuote;
                        }
                        "\"" => {
                            st.j = st.j + 1;
                            st.status = TokenStatus::InString;
                        }
                        "-" => {
                            match &self.input[st.j+1..st.j+2] { 
                                "." | "0" | "1" | "2" | "3" 
                                    | "4" | "5" | "6" | "7" 
                                    | "8" | "9" => {
                                    st.j = st.j + 1;
                                    st.status = TokenStatus::InNum;
                                }
                                _ => {
                                    st.j = st.j + 1;
                                    if let Some(ref f ) = self.map.borrow().get(&self.input[st.i..st.j]) {
                                        self.tokens.borrow_mut().push(f(&self.input[st.i..st.j]));
                                    } else {
                                        self.tokens.borrow_mut().push(self.new_token("literal", &self.input[st.i..st.j]));    
                                    }
                                    st.status = TokenStatus::EndToken;
                                }
                            }
                        }
                        "." => {
                            st.j = st.j + 1;
                            if st.j >= self.input.len() {
                                self.tokens.borrow_mut().push(self.new_token("end", &self.input[st.i..st.j]));
                                st.status = TokenStatus::EndToken;
                            } else {
                                st.status = TokenStatus::InNum;
                            }
                        }
                        "0" | "1" | "2" | "3" 
                            | "4" | "5" | "6" | "7" 
                            | "8" | "9" => {
                            st.j = st.j + 1;
                            st.status = TokenStatus::InNum;
                        }
                        _ => {
                            if let Some(ref f ) = self.map.borrow().get(&self.input[st.j..st.j+1]) {
                                self.tokens.borrow_mut().push(f(&self.input[st.j..st.j+1]));
                                st.j = st.j + 1;
                                st.status = TokenStatus::EndToken;
                            } else {
                                st.j = st.j + 1;
                                st.status = TokenStatus::InToken;
                            }
                        }
                    }
                }
    
                TokenStatus::InNum => {
                    match &self.input[st.j..st.j+1] {
                        " " | "\t" | "\n" | "'" | "(" | ")" | "\"" => {
                            self.tokens.borrow_mut().push(self.new_token("num", &self.input[st.i..st.j]));    
                            st.j = st.j - 1;
                            st.status = TokenStatus::EndToken;
                        }
                        "." | "0" | "1" | "2" | "3" 
                            | "4" | "5" | "6" | "7" 
                            | "8" | "9" => { st.j = st.j + 1; }
                        _ => {
                            println!("parse num error");
                            unreachable!();
                        }
                    }
                }
    
    
                TokenStatus::InToken => {
                    match &self.input[st.j..st.j+1] {
                        " " | "\t" | "\n" | "'" | "(" | ")" | "\"" => {
                            let s =&self.input[st.i..st.j];
                            if let Some(ref f ) = self.map.borrow().get(s) {
                                 self.tokens.borrow_mut().push(f(s));
                            } else {
                                 self.tokens.borrow_mut().push(self.new_token("literal", s));    
                            }
                            st.j = st.j - 1;
                            st.status = TokenStatus::EndToken;
                        }
                        _ => { st.j = st.j + 1; }
                    }
                }
    
                TokenStatus::InString => {
                    match &self.input[st.j..st.j+1] {
                        "\""  => {
                            let s = &self.input[st.i..st.j];    
                            self.tokens.borrow_mut().push(self.new_token("string", s));    
                            st.status = TokenStatus::EndToken;
                        }
                        _ => { st.j = st.j + 1; }
                    }
                }
    
                TokenStatus::InQuote => {
                    let mut ret = false;
                    match &self.input[st.j..st.j+1] {
                        " " | "\t" | "\n" => {
                            if  st.inquotepar == 0 {
                                ret = true;
                            } else {
                                st.j = st.j + 1;
                            }
                        }
                        "(" => {
                            st.inquotepar = st.inquotepar + 1; 
                            st.j = st.j + 1; 
                        }
                        ")" => {
                            if  st.inquotepar == 0 {
                                ret = true;
                            } else {
                                st.inquotepar = st.inquotepar - 1; 
                                st.j = st.j + 1; 
                            }
                        }
                        _ => { st.j = st.j + 1; }
                    }
                    if ret {
                        
                        let s = &self.input[st.i..st.j];
                        self.tokens.borrow_mut().push(self.new_token("string", s));    
                        
                        st.j = st.j - 1;
                        st.status = TokenStatus::EndToken;
                    }
                }
                TokenStatus::EndToken => {
                    //v.push(token.take().unwrap());    
                    st.j = st.j + 1;
                    st.i = st.j;
                    st.status = TokenStatus::Init;
                    break;
                }
            }
        }
    }
}


   

#[derive(Debug)]
struct EndToken {
    lbp: u8,
}


impl<S: Symbol> Token<S> for EndToken {
    fn lbp(&self) -> u8 {
        self.lbp
    }
}

   
#[derive(Debug)]
struct LiteralToken {
    code: String,
    lbp: u8,
}
 

impl Token<StaticSymbol> for LiteralToken {
    fn led(&mut self, _this: PrattBox<StaticSymbol>, _pratt: &Pratt<StaticSymbol>, _left: PrattBox<StaticSymbol>) -> PrattBox<StaticSymbol> {
        unreachable!();
    }
    fn nud(&mut self, this: PrattBox<StaticSymbol>, _pratt: &Pratt<StaticSymbol>) -> PrattBox<StaticSymbol> { 
        this
    }
    fn lbp(&self) -> u8 {
        self.lbp
    }
}

#[derive(Debug)]
struct StringToken {
    code: String,
    lbp: u8,
}
 
impl Token<StaticSymbol> for StringToken {
    fn nud(&mut self, this: PrattBox<StaticSymbol>, _pratt: &Pratt<StaticSymbol>) -> PrattBox<StaticSymbol> { 
        this
    }
    fn lbp(& self) -> u8 {
        self.lbp
    }
}


#[derive(Debug)]
struct NumToken {
    code: String,
    val: i64,
    lbp: u8,
}


impl Token<StaticSymbol> for NumToken  {
    fn nud(&mut self, this: PrattBox<StaticSymbol>, _pratt: &Pratt<StaticSymbol>) -> PrattBox<StaticSymbol> { 
        this
    }
    fn lbp(&self) -> u8 {
        self.lbp
    }
}

#[derive( Debug)]
struct PlusToken {
    code: String,
    left: Option<PrattBox<StaticSymbol>>,
    right: Option<PrattBox<StaticSymbol>>,
    lbp: u8,
}
 


impl Token<StaticSymbol> for PlusToken  {
    fn led(&mut self, this: PrattBox<StaticSymbol>, pratt: &Pratt<StaticSymbol>, left: PrattBox<StaticSymbol>) -> PrattBox<StaticSymbol> {
        // 'this' is only passed to be returned if needed
        // self is the mutable content of 'this'
        // we cannot pass self as immutable, then extract it as mut from 'this'
        // because the content is already borrowed by the calling function as self
        self.left = Some(left);
        self.right = Some(pratt.parse(0)); 
        this
    }
    fn lbp(& self) -> u8 {
        self.lbp
    }
}


#[derive( Debug)]
struct MultToken {
    code: String,
    left: Option<PrattBox<StaticSymbol>>,
    right: Option<PrattBox<StaticSymbol>>,
    lbp: u8,
}

   
impl Token<StaticSymbol> for MultToken  {
    fn led(&mut self, this: PrattBox<StaticSymbol>, pratt: &Pratt<StaticSymbol>, left: PrattBox<StaticSymbol>) -> PrattBox<StaticSymbol> {
        self.left = Some(left);
        self.right = Some(pratt.parse(0)); 
        this
    }
    fn lbp(& self) -> u8 {
        self.lbp
    }
}
 
#[derive(Debug)]
enum StaticSymbol {
    EndSymbol(EndToken),
    LiteralSymbol(LiteralToken),
    StringSymbol(StringToken),
    NumSymbol(NumToken),
    PlusSymbol(PlusToken),
    MultSymbol(MultToken),
}

use self::StaticSymbol::*;
 
fn newlit(code: &str) -> PrattBox<StaticSymbol> {
    prattbox!(StaticSymbol::LiteralSymbol( LiteralToken { code: String::from(code), 
                         lbp: 0,  }))  
}
fn newstring(code: &str) -> PrattBox<StaticSymbol> {
    prattbox!(StaticSymbol::StringSymbol( StringToken { code: String::from(code), 
                         lbp: 0,  }))  
}
fn newnum(code: &str) -> PrattBox<StaticSymbol> {
    prattbox!(StaticSymbol::NumSymbol( NumToken { code: String::from(code), 
                         lbp: 0,  val: i64::from_str_radix(code, 10).ok().unwrap()}))  
}
fn newplus(code: &str) -> PrattBox<StaticSymbol> {
    prattbox!(StaticSymbol::PlusSymbol( PlusToken { code: String::from(code), 
                         lbp: 20, left: None, right: None }))
}
fn newmult(code: &str) -> PrattBox<StaticSymbol> {
    prattbox!(StaticSymbol::MultSymbol( MultToken { code: String::from(code), 
                         lbp: 30, left: None, right: None }))
}

impl Symbol for StaticSymbol {
    fn token(&mut self) -> &mut dyn Token<StaticSymbol> {
        match *self {
            EndSymbol(ref mut t) =>  (t ) ,
            LiteralSymbol(ref mut t) =>  (t ) ,
            StringSymbol(ref mut t) =>  (t ) ,
            NumSymbol(ref mut t) => (t ) ,
            PlusSymbol(ref mut t) => (t ),
            MultSymbol(ref mut t) => (t ),
            // _ => { unreachable!(); }
        }
    }
}

#[cfg(feature="gc3c")]
impl Mark for StaticSymbol {
    fn mark(&self, gc: &mut InGcEnv) {
        match *self {
            PlusSymbol(ref t) => {
                if let Some(ref left) = t.left {
                    left.mark_grey(gc);
                }
                if let Some(ref right) = t.right {
                    right.mark_grey(gc);
                }
            },
            MultSymbol(ref t) => {
                if let Some(ref left) = t.left {
                    left.mark_grey(gc);
                }
                if let Some(ref right) = t.right {
                    right.mark_grey(gc);
                }
            },
            _ => {},
        }
    }
}
 

#[test] 
fn test_static() {
    let program = "1 + 2 * 3 .";
    let tokenizer = StringTokenizer::new(String::from(program));
    tokenizer.register_token("end", Box::new(|_s| { prattbox!(EndSymbol(EndToken{lbp:0}))}));    
    tokenizer.register_token("string", Box::new(newstring));
    tokenizer.register_token("literal", Box::new(newlit));
    tokenizer.register_token("num", Box::new(newnum));
    tokenizer.register_token("+", Box::new(newplus));
    tokenizer.register_token("*", Box::new(newmult));
    let parser = Pratt::new(Box::new(tokenizer));
    let ast = parser.pparse();
    match *ast.borrow_mut() {
        PlusSymbol( PlusToken { left : ref l, right: ref r, .. }) => {
            #[cfg(feature="gc3c")]
            let left = l.unwrap();
            #[cfg(not(feature="gc3c"))]
            let left = l.clone().unwrap();
            match *left.borrow_mut() {
                NumSymbol(NumToken { val: v, .. }) => {
                    assert_eq!(v, 1);
                },
                _ => {
                    assert!(false, "1 not found");
                }
            };
            #[cfg(feature="gc3c")]
            let right = r.unwrap();
            #[cfg(not(feature="gc3c"))]
            let right = r.clone().unwrap();
            match *right.borrow_mut() {
                MultSymbol(MultToken { left : ref l, right: ref r, .. }) => {
                    #[cfg(feature="gc3c")]
                    let left = l.unwrap();
                    #[cfg(not(feature="gc3c"))]
                    let left = l.clone().unwrap();
                    match *left.borrow_mut() {
                        NumSymbol(NumToken { val: v, .. }) => {
                            assert_eq!(v, 2);
                        },
                        _ => {
                            assert!(false, "2 not found");
                        }
                    };
                    #[cfg(feature="gc3c")]
                    let right = r.unwrap();
                    #[cfg(not(feature="gc3c"))]
                    let right = r.clone().unwrap();
                    match *right.borrow_mut() {
                        NumSymbol(NumToken { val: v, .. }) => {
                            assert_eq!(v, 3);
                        },
                        _ => {
                            assert!(false, "3 not found");
                        }
                    };

                },
                _ => {
                    assert!(false, "mult not found");
                }
            };
        },
        _ => {
            assert!(false, "plus not found");
        }
    };
    #[cfg(feature="gc3c")]
    gc::finalize();
}


  
 #[test] 
 fn test_dynamic() {
     let program = "1 + 2 * 3 .";
     let tokenizer = StringTokenizer::new(String::from(program));
     tokenizer.register_token("end", 
                              Box::new(|s| { 
                                  prattbox!(
                                      DynamicSymbol { 
                                          token: DynamicToken {
                                                    code: String::from(s), 
                                                     lbp:0,
                                                     children: vec![],
                                                     fnud: Rc::new(|_se, _this, _pratt| { unreachable!(); }),
                                                     fled: Rc::new(|_se, _this, _pratt, _left| { unreachable!(); }),
                                                 }
                                      }
                                  )
                              }
                            ));    

     tokenizer.register_token("string", 
                              Box::new(|s| { 
                                  prattbox!(
                                      DynamicSymbol { 
                                          token: DynamicToken {
                                                     code: String::from(s), 
                                                     lbp:0,
                                                     children: vec![],
                                                     fnud: Rc::new(|_se, this, _pratt| { this }),
                                                     fled: Rc::new(|_se, _this, _pratt, _left| { unreachable!(); }),
                                                 }
                                      }
                                  )
                              }
                            )); 
     
     tokenizer.register_token("literal",
                              Box::new(|s| { 
                                  prattbox!(
                                      DynamicSymbol { 
                                          token: DynamicToken {
                                                     code: String::from(s), 
                                                     lbp:0,
                                                     children: vec![],
                                                     fnud: Rc::new(|_se, this, _pratt| { this }),
                                                     fled: Rc::new(|_se, _this, _pratt, _left| { unreachable!(); }),
                                                 }
                                      }
                                  )
                              }
                            )); 
     

     tokenizer.register_token("num",
                              Box::new(|s| { 
                                  prattbox!(
                                      DynamicSymbol { 
                                          token: DynamicToken {
                                                     code: String::from(s), 
                                                     lbp:0,
                                                     children: vec![],
                                                     fnud: Rc::new(|_se, this, _pratt| { this }),
                                                     fled: Rc::new(|_se, _this, _pratt, _left| { unreachable!(); }),
                                                 }
                                      }
                                  )
                              }
                            )); 
     
     tokenizer.register_token("+",
                              Box::new(|s| { 
                                  prattbox!(
                                      DynamicSymbol { 
                                          token: DynamicToken {
                                                     code: String::from(s), 
                                                     lbp: 20,
                                                     children: vec![],
                                                     fnud: Rc::new(|_se, _this, _pratt| { unreachable!(); }),
                                                     fled: Rc::new(|se, this, pratt, left| { 
                                                                       se.add_child(left);
                                                                       se.add_child(pratt.parse(0));
                                                                       this
                                                                   }),
                                                 }
                                      }
                                  )
                              }
                            )); 
     

     tokenizer.register_token("*",
                              Box::new(|s| { 
                                  prattbox!(
                                      DynamicSymbol { 
                                          token: DynamicToken {
                                                     code: String::from(s), 
                                                     lbp: 30,
                                                     children: vec![],
                                                     fnud: Rc::new(|_se, _this, _pratt| { unreachable!(); }),
                                                     fled: Rc::new(|se, this, pratt, left| { 
                                                                       se.add_child(left);
                                                                       se.add_child(pratt.parse(0));
                                                                       this
                                                                   }),
                                                 }
                                      }
                                  )
                              }
                            )); 
     

     let parser = Pratt::new(Box::new(tokenizer));
     let ast = parser.pparse();
      match *ast.borrow_mut() {
        DynamicSymbol{ token: DynamicToken { ref code, ref children, .. }} => {
            assert_eq!(&"+", code);
            let left = children.get(0).unwrap();
            match *left.borrow_mut() {
                DynamicSymbol{ token: DynamicToken { ref code, .. }} => {
                    assert_eq!(code, &"1");
                },
            };
            let right = children.get(1).unwrap();
            match *right.borrow_mut() {
                DynamicSymbol{ token: DynamicToken { ref code, ref children, .. }} => {
                    assert_eq!(code, &"*");
                    let left = children.get(0).unwrap();
                    match *left.borrow_mut() {
                        DynamicSymbol{ token: DynamicToken { ref code, .. }} => {
                            assert_eq!(code, &"2");
                        },
                    };
                    let right = children.get(1).unwrap();
                    match *right.borrow_mut() {
                        DynamicSymbol{ token: DynamicToken { ref code, .. }} => {
                            assert_eq!(code, &"3");
                        },
                    };

                },
            };
        },
    };

    #[cfg(feature="gc3c")]
    gc::finalize();
}
