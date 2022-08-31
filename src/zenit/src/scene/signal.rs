use super::node::NodeRef;

pub struct Signal<P1: Copy = ()> {
    listeners: Vec<Box<dyn FnOnce(P1)>>,
}

pub trait IntoSignalHandler<P1: Copy = ()> {
    fn into_sh(self) -> Box<dyn FnOnce(P1)>;
}

impl<P1: Copy> IntoSignalHandler<P1> for Box<dyn FnOnce(P1)> {
    fn into_sh(self) -> Box<dyn FnOnce(P1)> {
        self
    }
}

impl IntoSignalHandler<()> for Box<dyn FnOnce()> {
    fn into_sh(self) -> Box<dyn FnOnce(())> {
        Box::new(move |_| (self)())
    }
}
