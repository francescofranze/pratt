use std::fmt;
use super::{PrattBox, Token, Symbol, Pratt};
use super::InspectAST;
#[cfg(feature="gc3c")]
use gc3c::{Mark,InGcEnv};
#[cfg(not(feature="gc3c"))]
use super::Mark;
use std::rc::Rc;
 
pub struct DynamicToken<S: Symbol>  {
    pub code: String,
    pub children: Vec<PrattBox<S>>,
    pub lbp: u8,
    pub fnud: Rc<Fn(&mut DynamicToken<S>, PrattBox<S>, &Pratt<S>)->PrattBox<S>>,
    pub fled: Rc<Fn(&mut DynamicToken<S>, PrattBox<S>, &Pratt<S>, PrattBox<S>)->PrattBox<S>>,
}

pub struct DynamicSymbol {
    token: DynamicToken<DynamicSymbol>,
}

impl Symbol for DynamicSymbol {
    fn token(&mut self) -> &mut Token<DynamicSymbol> {
        &mut self.token
    }
}

impl fmt::Debug for DynamicSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.token.fmt(f)
    }
}

impl<S: Symbol> fmt::Debug for DynamicToken<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "dynamic {}", self.code));
        for c in &self.children {
            try!(write!(f, "child: {:?}", c));
        }
        write!(f,"")
    }
}

impl<S: Symbol> DynamicToken<S> {
    pub fn add_child(&mut self, child: PrattBox<S>) {
        self.children.push(child);
    }
    pub fn get_child(&self, i: usize) -> Option<&PrattBox<S>> {
        self.children.get(i)
    }
}


impl<S: Symbol> Token<S> for DynamicToken<S>  {
    fn nud(&mut self, this: PrattBox<S>, pratt: &Pratt<S>) -> PrattBox<S>
    {
        (self.fnud.clone())(self, this, pratt)
    }
    fn led(&mut self, this: PrattBox<S>, pratt: &Pratt<S>, left: PrattBox<S>) -> PrattBox<S>
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
