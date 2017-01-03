use std::fmt;
use super::{PrattBox, Token, Symbol, Pratt};
use super::InspectAST;
#[cfg(feature="gc3c")]
use gc3c::{Mark,InGcEnv};
#[cfg(not(feature="gc3c"))]
use super::Mark;
use std::rc::Rc;
 
pub struct DynamicToken  {
    pub code: String,
    pub children: Vec<PrattBox<Symbol>>,
    pub lbp: u8,
    pub fnud: Rc<Fn(&mut DynamicToken, PrattBox<Symbol>, &Pratt)->PrattBox<DynamicSymbol>>,
    pub fled: Rc<Fn(&mut DynamicToken, PrattBox<Symbol>, &Pratt, PrattBox<Symbol>)->PrattBox<DynamicSymbol>>,
}

pub struct DynamicSymbol {
    token: DynamicToken,
}

impl Symbol for DynamicSymbol {
    fn token(&mut self) -> &mut Token {
        &mut self.token
    }
}

impl fmt::Debug for DynamicSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.token.fmt(f)
    }
}

impl fmt::Debug for DynamicToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "dynamic {}", self.code));
        for c in &self.children {
            try!(write!(f, "child: {:?}", c));
        }
        write!(f,"")
    }
}

impl DynamicToken {
    pub fn add_child(&mut self, child: PrattBox<Symbol>) {
        self.children.push(child);
    }
    pub fn get_child(&self, i: usize) -> Option<&PrattBox<Symbol>> {
        self.children.get(i)
    }
}


impl Token for DynamicToken  {
    fn nud(&mut self, this: PrattBox<Symbol>, pratt: &Pratt) -> PrattBox<Symbol>
    {
        (self.fnud.clone())(self, this, pratt)
    }
    fn led(&mut self, this: PrattBox<Symbol>, pratt: &Pratt, left: PrattBox<Symbol>) -> PrattBox<Symbol>
    {
        //let fled = self.fled.clone();
        self.fled.clone()(self, this, pratt, left)
    }
    fn lbp(&self) -> u8 {
        self.lbp
    }
}
#[cfg(not(feature="gc3c"))]
impl Mark for DynamicSymbol {}

#[cfg(feature="gc3c")]
impl Mark for DynamicSymbol {
    fn mark(&self, gc: &mut InGcEnv) {
    }
}
