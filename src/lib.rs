use std::fmt;
use std::fmt::Write;


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



pub trait Token : fmt::Debug+InspectAST+Mark {
    fn led(&mut self, this: PrattBox<Token>, pratt: &Pratt, left: PrattBox<Token>) -> PrattBox<Token> {
        let _ = this;
        let _ = pratt;
        let _ = left;
        unreachable!();
    }
    fn nud(&mut self, this: PrattBox<Token>, pratt: &Pratt) -> PrattBox<Token> {
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

pub trait Tokenizer {
    fn advance(&self);
    fn current(& self) -> Option<PrattBox<Token>>;
}

pub struct Pratt {
    tokenizer: Box<Tokenizer>,
}

impl Pratt {
    fn new(tokenizer: Box<Tokenizer>) -> Pratt {
        Pratt { tokenizer: tokenizer }
    }

    fn advance(&self) {
        self.tokenizer.advance()
    }
    fn current(& self) -> Option<PrattBox<Token>> {
        self.tokenizer.current()
    }

    fn parse(&self, rbp: u8) -> PrattBox<Token>  {
        let mut t = self.current().unwrap();
        self.advance();
        let mut this = t.clone();
        let mut left = t.borrow_mut().nud(this, self);
        let mut lookahead = self.current().unwrap(); 
        while rbp < lookahead.borrow().lbp() {
            t = lookahead;
            self.advance();
            this = t.clone();
            left = t.borrow_mut().led(this, self, left);
            lookahead = self.current().unwrap();
        }
        return left;
    }

    fn pparse(& self) -> PrattBox<Token>  {
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
    use super::{PrattBox, Token, Pratt, Tokenizer, InspectAST, Mark};
    
    #[cfg(feature="gc3c")]
    macro_rules! prattbox {
        ($expression:expr) => (
            gc::new_gc($expression)
        )
    }
    
    

    type FnewToken = Box<Fn(&str) -> PrattBox<Token>>;

   
    
    #[derive(Debug)]
    struct End {
        lbp: u8,
    }
    
    impl End {
        fn new() -> PrattBox<Token> {
            prattbox!(End {  lbp: 0 })
        }
    }
    
    impl Token for End {
        fn lbp(&self) -> u8 {
            self.lbp
        }
    }
    
    impl InspectAST for End {}
    impl Mark for End {}
    
    
    #[derive(Debug)]
    struct LiteralToken {
        code: String,
        lbp: u8,
    }
    
    impl LiteralToken {
        fn new(code: &str) -> PrattBox<Token> {
            prattbox!(LiteralToken { code: String::from(code), lbp: 0 })
        }
    }
    
    
    impl Token for LiteralToken {
        fn led(&mut self, this: PrattBox<Token>, pratt: &Pratt, left: PrattBox<Token>) -> PrattBox<Token> {
            unreachable!();
        }
        fn nud(&mut self, this: PrattBox<Token>, pratt: &Pratt) -> PrattBox<Token> { 
            this
        }
        fn lbp(&self) -> u8 {
            self.lbp
        }
    }
    impl InspectAST for LiteralToken {}
    
    #[derive(Debug)]
    struct StringToken {
        code: String,
        lbp: u8,
    }
    
    impl StringToken {
        fn new(code: &str) -> PrattBox<Token> {
            prattbox!(StringToken { code: String::from(code), lbp: 0 })
        }
    }
    impl Token for StringToken {
        fn nud(&mut self, this: PrattBox<Token>, pratt: &Pratt) -> PrattBox<Token> { 
            this
        }
        fn lbp(&self) -> u8 {
            self.lbp
        }
    }
    impl InspectAST for StringToken {}
    
    
    #[derive(Debug)]
    struct NumToken {
        code: String,
        val: i64,
        lbp: u8,
    }
    
    impl NumToken {
        fn new(code: &str) -> PrattBox<Token> {
            prattbox!(
                NumToken { code: String::from(code), 
                           lbp: 0,
                           val: i64::from_str_radix(code, 10).ok().unwrap()})  
        }
    }
    impl Token for NumToken  {
        fn nud(&mut self, this: PrattBox<Token>, pratt: &Pratt) -> PrattBox<Token> { 
            this
        }
        fn lbp(&self) -> u8 {
            self.lbp
        }
    }
    impl InspectAST for NumToken {}
    
    #[derive( Debug)]
    struct PlusToken {
        code: String,
        left: Option<PrattBox<Token>>,
        right: Option<PrattBox<Token>>,
        lbp: u8,
    }
    
    
    impl PlusToken {
        fn new(code: &str) -> PrattBox<Token> {
            prattbox!(PlusToken { code: String::from(code), 
                                lbp: 20, left: None, right: None })
        }
    }
    
    impl Token for PlusToken  {
        fn led(&mut self, this: PrattBox<Token>, pratt: &Pratt, left: PrattBox<Token>) -> PrattBox<Token>
        {
            self.left = Some(left);
            self.right = Some(pratt.parse(0)); 
            this
        }
        fn lbp(&self) -> u8 {
            self.lbp
        }
    }
    impl InspectAST for PlusToken {}
    
    
    #[derive( Debug)]
    struct MultToken {
        code: String,
        left: Option<PrattBox<Token>>,
        right: Option<PrattBox<Token>>,
        lbp: u8,
    }
    
    
    impl MultToken {
        fn new(code: &str) -> PrattBox<Token> {
            prattbox!(MultToken { code: String::from(code), 
                                lbp: 30, left: None, right: None })
        }
    }
    impl InspectAST for MultToken {}
    
    impl Token for MultToken  {
        fn led(&mut self, this: PrattBox<Token>, pratt: &Pratt, left: PrattBox<Token>) -> PrattBox<Token>
        {
            self.left = Some(left);
            self.right = Some(pratt.parse(0)); 
            this
        }
        fn lbp(&self) -> u8 {
            self.lbp
        }
    }
    
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
    
    struct StringTokenizer {
        input: String,
        tokens : RefCell<Vec<PrattBox<Token>>>,
        map: RefCell<HashMap<&'static str, FnewToken>>,
        st: RefCell<TokenizerStatus>,
    }
    
    impl StringTokenizer {
        fn new(input: String) -> StringTokenizer {
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
        fn new_token(&self, typ: &str, s: &str) -> PrattBox<Token> {
            if let Some(ref f ) = self.map.borrow().get(typ) {
                f(s)
            } else {
                unreachable!("cannot handle token");
            }
        }
    
        fn register_token(& self, s: &'static str, f: FnewToken) {
            self.map.borrow_mut().insert(s, f);
        }
    }
    
    impl Tokenizer for StringTokenizer {
    
        fn current(&self) -> Option<PrattBox<Token>> {
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
                                    self.tokens.borrow_mut().push(End::new());    
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
    impl Mark for PlusToken {}
    impl Mark for MultToken {}
    
    #[test] 
    fn test_static() {
        let program = "1 + 2 * 3 .";
        let tokenizer = StringTokenizer::new(String::from(program));
        tokenizer.register_token("string", Box::new(StringToken::new));
        tokenizer.register_token("literal", Box::new(LiteralToken::new));
        tokenizer.register_token("num", Box::new(NumToken::new));
        tokenizer.register_token("+", Box::new(PlusToken::new));
        tokenizer.register_token("*", Box::new(MultToken::new));
        let parser = Pratt::new(Box::new(tokenizer));
        let ast = parser.pparse();
        let q= ast as PrattBox<MultToken>;
        match *ast.borrow() {
            MultToken { left : ref l, right: ref r, .. } => {
            },
            _ => {
                assert!(false, "mult not found");
            }
        }
        gc::finalize();
    }
    

}
