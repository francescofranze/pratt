// use std::cmp::PartialEq;

pub mod pratt {
    use std::cell::RefCell;
    use std::slice::Iter;
    use std::io::Read;
    use std::collections::*;
    use std::rc::*;
    use std::fmt;
    //  we need here a smart box like rc, or box, which borrowable mutable content
    //  so Rc with RefCell
    //
    pub type PrattBox<T> = Rc<RefCell<T>>;
    pub type FnewToken = Box<Fn(&str) -> PrattBox<Token>>;

    
    #[macro_export]
    macro_rules! prattbox {
        ($expr:expr) => (
            Rc::new(RefCell::new($expr))
        )
    }
    
    
    pub trait Token : fmt::Debug+InspectAST {
        fn led(& self, pratt: & Pratt, left: PrattBox<Token>) -> PrattBox<Token> {
            unreachable!();
        }
        fn nud(& self, pratt: & Pratt) -> PrattBox<Token> {
            unreachable!();
        }
        fn lbp(&self) -> u8 ;
    }
    
    impl<T: Token+?Sized> Token for PrattBox<T> {
        fn led(& self, pratt: & Pratt, left: PrattBox<Token>) -> PrattBox<Token> {
            self.borrow().led(pratt, left)
        }
        fn nud(& self, pratt: & Pratt) -> PrattBox<Token> {
            self.borrow().nud(pratt)
        }
        fn lbp(&self) -> u8 {
            self.borrow().lbp()
        }
    }
    
    pub trait InspectAST where Self: fmt::Debug {
        fn inspect(&self) {
            println!("{:?}", self);
        }
    }
    
    impl<T: InspectAST+?Sized> InspectAST for PrattBox<T> {
        fn inspect(&self)  {
            self.borrow().inspect();
        }
    }
    
    
    pub trait Tokenizer  {
        fn advance(&self);
        fn current(& self) -> Option<PrattBox<Token>> ;
        fn register_token(&self, &'static str, f: FnewToken); 
    }
    
    pub struct Pratt {
        tokenizer: Box<Tokenizer>,
    }
    
    impl Pratt    {
        pub fn new(tokenizer: Box<Tokenizer>) -> Self {
            Pratt { tokenizer: tokenizer }
        }
    
        pub fn advance(&self) {
            self.tokenizer.advance();
        }
        pub fn current(&self) -> Option<PrattBox<Token>> {
            self.tokenizer.current()
        }
    
        pub fn parse(&self, rbp: u8) -> PrattBox<Token> {
            let mut t = self.current().unwrap();
            self.advance();
            let mut left = t.nud(self);
            let mut lookahead = self.current().unwrap(); 
            while rbp < lookahead.lbp() {
                t = lookahead;
                self.advance();
                left = t.led(self, left);
                lookahead = self.current().unwrap();
            }
            return left;
        }
    
        pub fn pparse(& self) -> PrattBox<Token> {
            self.advance();
            self.parse(0)
        }
    }
}

pub mod dyn;
   
