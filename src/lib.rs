
#[cfg(not(feature="gc3c"))]
#[macro_use]
pub mod rcbox;
#[cfg(not(feature="gc3c"))]
pub use rcbox::PrattBox;

#[cfg(feature="gc3c")]
extern crate gc3c;
#[cfg(feature="gc3c")]
use gc3c::{Gc, Mark};
#[cfg(feature="gc3c")]
pub type PrattBox<T> = Gc<T>;

#[cfg(feature="gc3c")]
#[macro_export]
macro_rules! prattbox {
    ($expression:expr) => (
        gc::new_gc($expression)
    )
}

#[cfg(feature="gc3c")]
pub trait Symbol: Mark  {
    fn token(&mut self) -> &mut dyn Token<Self>;
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


#[cfg(not(feature="gc3c"))]
pub trait Symbol  {
    fn token(&mut self) -> &mut dyn Token<Self>;
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
    fn led(&mut self, _this: PrattBox<S>, _pratt: &Pratt<S>, _left: PrattBox<S>) -> PrattBox<S> {
        unreachable!();
    }
    fn nud(&mut self, _this: PrattBox<S>, _pratt: &Pratt<S>) -> PrattBox<S> {
        unreachable!();
    }
    fn lbp(&self) -> u8 ;
}
    


pub trait Tokenizer<S: Symbol> {
    fn advance(&self);
    fn current(& self) -> Option<PrattBox<S>>;
}

pub struct Pratt<S: Symbol> {
    tokenizer: Box<dyn Tokenizer<S>>,
}

impl<S: Symbol> Pratt<S> {
    pub fn new(tokenizer: Box<dyn Tokenizer<S>>) -> Pratt<S> {
        Pratt { tokenizer: tokenizer }
    }

    pub fn advance(&self) {
        self.tokenizer.advance()
    }
    pub fn current(& self) -> Option<PrattBox<S>> {
        self.tokenizer.current()
    }

    fn nud(&self, this: PrattBox<S>) -> PrattBox<S> {
        this.borrow_mut().nud(this.clone(), self)
    }
    fn led(&self, this: PrattBox<S>, left: PrattBox<S>) -> PrattBox<S> {
        this.borrow_mut().led(this.clone(), self, left)
    }

    pub fn parse(&self, rbp: u8) -> PrattBox<S>  {
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

    pub fn pparse(& self) -> PrattBox<S>  {
        self.advance();
        self.parse(0)
    }
}

pub mod dyn;
