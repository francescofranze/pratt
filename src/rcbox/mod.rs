use std::cell::RefCell;
use std::rc::Rc;
//  we need here a smart box like rc, or box, which borrowable mutable content
//  so Rc with RefCell
//
pub type PrattBox<T> = Rc<RefCell<T>>;

    
#[macro_export]
macro_rules! prattbox {
    ($expr:expr) => (
        Rc::new(RefCell::new($expr))
    )
}
