use std::fmt;
use super::{PrattBox, Token, Symbol, Pratt};
#[cfg(feature="gc3c")]
use gc3c::{Mark,InGcEnv};
#[cfg(not(feature="gc3c"))]
use super::Mark;
use std::rc::Rc;
 
pub struct DynamicToken  {
    pub code: String,
    pub children: Vec<PrattBox<DynamicSymbol>>,
    pub lbp: u8,
    pub fnud: Rc<Fn(&mut DynamicToken, PrattBox<DynamicSymbol>, &Pratt<DynamicSymbol>)->PrattBox<DynamicSymbol>>,
    pub fled: Rc<Fn(&mut DynamicToken, PrattBox<DynamicSymbol>, &Pratt<DynamicSymbol>, PrattBox<DynamicSymbol>)->PrattBox<DynamicSymbol>>,
}

pub struct DynamicSymbol {
    pub token: DynamicToken,
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
    pub fn add_child(&mut self, child: PrattBox<DynamicSymbol>) {
        self.children.push(child);
    }
    pub fn get_child(&self, i: usize) -> Option<&PrattBox<DynamicSymbol>> {
        self.children.get(i)
    }
}


impl Token<DynamicSymbol> for DynamicToken  {
    fn nud(&mut self, this: PrattBox<DynamicSymbol>, pratt: &Pratt<DynamicSymbol>) -> PrattBox<DynamicSymbol>
    {
        (self.fnud.clone())(self, this, pratt)
    }
    fn led(&mut self, this: PrattBox<DynamicSymbol>, pratt: &Pratt<DynamicSymbol>, left: PrattBox<DynamicSymbol>) -> PrattBox<DynamicSymbol>
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
        for &child in self.token.children.iter() {
            child.mark_grey(gc);
        }
    }
}
