use std::fmt;
use super::{PrattBox, Token, Pratt};
use super::InspectAST;
#[cfg(feature="gc3c")]
use gc3c::{Mark,InGcEnv};
#[cfg(not(feature="gc3c"))]
use super::Mark;
use std::rc::Rc;
 
pub struct DynamicToken  {
    pub code: String,
    pub children: Vec<PrattBox<Token>>,
    pub lbp: u8,
    pub fnud: Rc<Fn(&mut DynamicToken, PrattBox<Token>, &Pratt)->PrattBox<DynamicToken>>,
    pub fled: Rc<Fn(&mut DynamicToken, PrattBox<Token>, &Pratt, PrattBox<Token>)->PrattBox<DynamicToken>>,
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
    pub fn add_child(&mut self, child: PrattBox<Token>) {
        self.children.push(child);
    }
    pub fn get_child(&self, i: usize) -> Option<&PrattBox<Token>> {
        self.children.get(i)
    }
}

impl InspectAST for DynamicToken { }

impl Token for DynamicToken  {
    fn nud(&mut self, this: PrattBox<Token>, pratt: &Pratt) -> PrattBox<Token>
    {
        (self.fnud.clone())(self, this, pratt)
    }
    fn led(&mut self, this: PrattBox<Token>, pratt: &Pratt, left: PrattBox<Token>) -> PrattBox<Token>
    {
        //let fled = self.fled.clone();
        self.fled.clone()(self, this, pratt, left)
    }
    fn lbp(&self) -> u8 {
        self.lbp
    }
}
#[cfg(not(feature="gc3c"))]
impl Mark for DynamicToken {}

#[cfg(feature="gc3c")]
impl Mark for DynamicToken {
    fn mark(&self, gc: &mut InGcEnv) {
    }
}
