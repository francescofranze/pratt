use std::fmt;
use pratt::*;
use pratt::InspectAST;
 
pub struct DynamicToken {
    pub code: String,
    pub children: Vec<PrattBox<Token>>,
    pub lbp: u8,
    pub fnud: Box<Fn(&DynamicToken, &Pratt)->PrattBox<DynamicToken>>,
    pub fled: Box<Fn(&DynamicToken, &Pratt, PrattBox<Token>)->PrattBox<DynamicToken>>,
}


impl fmt::Debug for DynamicToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "dynamic {}", self.code));
        for c in &self.children {
            write!(f, "child: {:?}", c);
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
    fn nud(&self, pratt: &Pratt) -> PrattBox<Token>
    {
        (*self.fnud)(self, pratt)
    }
    fn led(&self, pratt: &Pratt, left: PrattBox<Token>) -> PrattBox<Token>
    {
        (*self.fled)(self, pratt, left)
    }
    fn lbp(&self) -> u8 {
        self.lbp
    }
}
