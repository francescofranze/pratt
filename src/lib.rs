#![feature(raw)]
use std::raw;
use std::mem;

use std::fmt;
use std::fmt::Write;
use std::cell::Ref;
use std::cell::RefMut;


#[cfg(not(feature="gc3c"))]
#[macro_use]
pub mod rcbox;
#[cfg(not(feature="gc3c"))]
use rcbox::PrattBox;
#[cfg(not(feature="gc3c"))]
pub trait Mark {}

#[cfg(feature="gc3c")]
extern crate gc3c;
#[cfg(feature="gc3c")]
use gc3c::{Gc, Mark, InGcEnv};
#[cfg(feature="gc3c")]
type PrattBox<T> = Gc<T>;


pub trait Symbol: Mark + fmt::Debug {
    fn token(&mut self) -> &mut Token<Self>;
    fn nud(&mut self, this: PrattBox<Self>, pratt: &Pratt<Self>) -> PrattBox<Self> where Self: Sized {
        self.token().nud(this, pratt)
    }
    fn led(&mut self, this: PrattBox<Self>, pratt: &Pratt<Self>, left: PrattBox<Self>) -> PrattBox<Self> where Self: Sized {
        self.token().led(this, pratt, left)
    }
    fn lbp(&mut self) -> u8 where Self: Sized {
        self.token().lbp()
    }
}


pub trait Token<S: Symbol>  {
    fn led(&mut self, this: PrattBox<S>, pratt: &Pratt<S>, left: PrattBox<S>) -> PrattBox<S> {
        let _ = this;
        let _ = pratt;
        let _ = left;
        unreachable!();
    }
    fn nud(&mut self, this: PrattBox<S>, pratt: &Pratt<S>) -> PrattBox<S> {
        let _ = this;
        let _ = pratt;
        unreachable!();
    }
    fn lbp(&self) -> u8 ;
}
    



pub trait InspectAST where Self: fmt::Debug {
    fn inspect(&self, f: &mut String) {
        write!(f, "{:?}", self);
    }
}

impl<T: InspectAST+?Sized+Mark> InspectAST for PrattBox<T> {
    fn inspect(&self, f: &mut String)  {
        self.borrow().inspect(f);
    }
}

pub trait Tokenizer<S: Symbol> {
    fn advance(&self);
    fn current(& self) -> Option<PrattBox<S>>;
}

pub struct Pratt<S: Symbol> {
    tokenizer: Box<Tokenizer<S>>,
}

impl<S: Symbol> Pratt<S> {
    fn new(tokenizer: Box<Tokenizer<S>>) -> Pratt<S> {
        Pratt { tokenizer: tokenizer }
    }

    fn advance(&self) {
        self.tokenizer.advance()
    }
    fn current(& self) -> Option<PrattBox<S>> {
        self.tokenizer.current()
    }

    fn nud(&self, this: PrattBox<S>) -> PrattBox<S> {
        this.borrow_mut().nud(this.clone(), self)
    }
    fn led(&self, this: PrattBox<S>, left: PrattBox<S>) -> PrattBox<S> {
        this.borrow_mut().led(this.clone(), self, left)
    }

    fn parse(&self, rbp: u8) -> PrattBox<S>  {
        let mut t = self.current().unwrap();
        self.advance();
        let mut left = self.nud(t);
        let mut lookahead = self.current().unwrap(); 
        while rbp < lookahead.borrow_mut().lbp() {
            t = lookahead;
            self.advance();
            left = self.led(t, left);
            lookahead = self.current().unwrap();
        }
        return left;
    }

    fn pparse(& self) -> PrattBox<S>  {
        self.advance();
        self.parse(0)
    }
}

pub mod dyn;

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::collections::HashMap;

    #[cfg(feature="gc3c")]
    use gc3c::{Gc, InGcEnv, gc};
    use super::{PrattBox, Token, Symbol, Pratt, Tokenizer, InspectAST, Mark};
    
    #[cfg(feature="gc3c")]
    macro_rules! prattbox {
        ($expression:expr) => (
            gc::new_gc($expression)
        )
    }
    
    

    type FnewToken<S> = Box<Fn(&str) -> PrattBox<S>>;

   
    
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
     
    
    impl<S: Symbol> Token<S> for LiteralToken {
        fn led(&mut self, this: PrattBox<S>, pratt: &Pratt<S>, left: PrattBox<S>) -> PrattBox<S> {
            unreachable!();
        }
        fn nud(&mut self, this: PrattBox<S>, pratt: &Pratt<S>) -> PrattBox<S> { 
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
     
    impl<S: Symbol> Token<S> for StringToken {
        fn nud(&mut self, this: PrattBox<S>, pratt: &Pratt<S>) -> PrattBox<S> { 
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
    

    impl<S: Symbol> Token<S> for NumToken  {
        fn nud(&mut self, this: PrattBox<S>, pratt: &Pratt<S>) -> PrattBox<S> { 
            this
        }
        fn lbp(& self) -> u8 {
            self.lbp
        }
    }
    
    #[derive( Debug)]
    struct PlusToken<S: Symbol> {
        code: String,
        left: Option<PrattBox<S>>,
        right: Option<PrattBox<S>>,
        lbp: u8,
    }
     

    
    impl<S: Symbol> Token<S> for PlusToken<S>  {
        fn led(&mut self, this: PrattBox<S>, pratt: &Pratt<S>, left: PrattBox<S>) -> PrattBox<S>
        {
            self.left = Some(left);
            self.right = Some(pratt.parse(0)); 
            this
        }
        fn lbp(& self) -> u8 {
            self.lbp
        }
    }
    
    
    #[derive( Debug)]
    struct MultToken<S: Symbol> {
        code: String,
        left: Option<PrattBox<S>>,
        right: Option<PrattBox<S>>,
        lbp: u8,
    }
    
       
    impl<S : Symbol> Token<S> for MultToken<S>  {
        fn led(&mut self, this: PrattBox<S>, pratt: &Pratt<S>, left: PrattBox<S>) -> PrattBox<S>
        {
            self.left = Some(left);
            self.right = Some(pratt.parse(0)); 
            this
        }
        fn lbp(& self) -> u8 {
            self.lbp
        }
    }
 
    #[derive(Debug)]
    pub enum StaticSymbol {
        EndSymbol(EndToken),
        LiteralSymbol(LiteralToken),
        StringSymbol(StringToken),
        NumSymbol(NumToken),
        PlusSymbol(PlusToken<StaticSymbol>),
        MultSymbol(MultToken<StaticSymbol>),
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
        fn token(&mut self) -> &mut Token<StaticSymbol> {
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
    impl Mark for StaticSymbol {}
 
   
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
            let mut tokens = self.tokens.borrow_mut();
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
    
    impl Mark for StringToken {}
    impl Mark for LiteralToken {}
    impl Mark for NumToken {}
    impl<S: Symbol> Mark for PlusToken<S> {}
    impl<S: Symbol> Mark for MultToken<S> {}
    
    #[test] 
    fn test_static() {
        let program = "1 + 2 * 3 .";
        let tokenizer = StringTokenizer::new(String::from(program));
        tokenizer.register_token("end", Box::new(|s| { prattbox!(EndSymbol(EndToken{lbp:0}))}));    
        tokenizer.register_token("string", Box::new(newstring));
        tokenizer.register_token("literal", Box::new(newlit));
        tokenizer.register_token("num", Box::new(newnum));
        tokenizer.register_token("+", Box::new(newplus));
        tokenizer.register_token("*", Box::new(newmult));
        let parser = Pratt::new(Box::new(tokenizer));
        let ast = parser.pparse();
        match *ast.borrow_mut() {
            PlusSymbol( PlusToken { left : ref l, right: ref r, .. }) => {
                let left = l.unwrap();
                match *left.borrow_mut() {
                    NumSymbol(NumToken { val: v, .. }) => {
                        assert_eq!(v, 1);
                    },
                    _ => {
                        assert!(false, "1 not found");
                    }
                };
                let right = r.unwrap();
                match *right.borrow_mut() {
                    MultSymbol(MultToken { left : ref l, right: ref r, .. }) => {
                    },
                    _ => {
                        assert!(false, "mult not found");
                    }
                };
            },
            _ => {
                assert!(false, "plus not found");
            }
        }
        gc::finalize();
    }
    

}
