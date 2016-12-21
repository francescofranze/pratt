use std::fmt;


#[cfg(not(feature="gc3c"))]
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

pub type FnewToken = Box<Fn(&str) -> PrattBox<Token>>;

pub trait Token : fmt::Debug+InspectAST+Mark {
    fn led(& self, pratt: & Pratt, left: PrattBox<Token>) -> PrattBox<Token> {
        unreachable!();
    }
    fn nud(& self, pratt: & Pratt) -> PrattBox<Token> {
        unreachable!();
    }
    fn lbp(&self) -> u8 ;
}



pub trait InspectAST where Self: fmt::Debug {
    fn inspect(&self) {
        println!("{:?}", self);
    }
}

impl<T: InspectAST+?Sized+Mark> InspectAST for PrattBox<T> {
    fn inspect(&self)  {
        self.borrow().inspect();
    }
}


pub trait Tokenizer  {
    fn advance(&self);
    fn current(& self) -> Option<PrattBox<Token>> ;
    fn register_token(&self, &'static str, f: FnewToken); 
}

pub trait Pratt     {

    fn advance(&self);
    fn current(& self) -> Option<PrattBox<Token>> ;

    fn parse(&self, rbp: u8) -> PrattBox<Token> where Self: Sized {
        let mut t = self.current().unwrap();
        self.advance();
        let mut left = t.borrow().nud(self);
        let mut lookahead = self.current().unwrap(); 
        while rbp < lookahead.borrow().lbp() {
            t = lookahead;
            self.advance();
            left = t.borrow().led(self, left);
            lookahead = self.current().unwrap();
        }
        return left;
    }

    fn pparse(& self) -> PrattBox<Token> where Self: Sized {
        self.advance();
        self.parse(0)
    }
}

pub mod dyn;
