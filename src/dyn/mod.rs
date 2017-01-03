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
    pub children: Vec<PrattBox<DynamicSymbol>>,
    pub lbp: u8,
    pub fnud: Rc<Fn(&mut DynamicToken, PrattBox<DynamicSymbol>, &Pratt<DynamicSymbol>)->PrattBox<DynamicSymbol>>,
    pub fled: Rc<Fn(&mut DynamicToken, PrattBox<DynamicSymbol>, &Pratt<DynamicSymbol>, PrattBox<DynamicSymbol>)->PrattBox<DynamicSymbol>>,
}

pub struct DynamicSymbol {
    token: DynamicToken,
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
    }
}

#[cfg(test)]
pub mod tests {
    use std::fmt;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::collections::HashMap;

    #[cfg(feature="gc3c")]
    use gc3c::{Gc, InGcEnv, gc, Mark};
    use super::{PrattBox, Token, Symbol, Pratt };
    use super::super::Tokenizer;
    use super::{DynamicToken, DynamicSymbol};

     type FnewToken<S> = Box<Fn(&str) -> PrattBox<S>>;


    
    #[cfg(feature="gc3c")]
    macro_rules! prattbox {
        ($expression:expr) => (
            gc::new_gc($expression)
        )
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
        }
    }


     
    #[test] 
    fn test_dynamic() {
        let program = "1 + 2 * 3 .";
        let tokenizer = StringTokenizer::new(String::from(program));
        tokenizer.register_token("end", Box::new(|s| { prattbox!(DynamicSymbol{ token: DynamicToken {
                                                                                            code: String::from(s), 
                                                                                            lbp:0,
                                                                                            children: vec![],
                                                                                            fnud: Rc::new(|se, this, pratt| { this }),
                                                                                            fled: Rc::new(|se, this, pratt, left| { this }),
                                                                                       }})}));    
        /*
        tokenizer.register_token("string", Box::new(newstring));
        tokenizer.register_token("literal", Box::new(newlit));
        tokenizer.register_token("num", Box::new(newnum));
        tokenizer.register_token("+", Box::new(newplus));
        tokenizer.register_token("*", Box::new(newmult));
        */
        let parser = Pratt::new(Box::new(tokenizer));
        let ast = parser.pparse();

    }
}
