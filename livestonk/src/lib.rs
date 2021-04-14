pub use livestonk_derive::Component as Component;

pub struct Livestonk {
}

pub trait Resolve<T: ?Sized> {

    fn resolve() -> Box<T>;
}

#[macro_export]
macro_rules! bind_to_instance {
    (dyn $a:ident,$b:expr) => {
        impl livestonk::Resolve<dyn $a> for livestonk::Livestonk {
            fn resolve() -> Box<dyn $a> {
                box $b
            }
        }
    };
}

#[macro_export]
macro_rules! bind {
    (dyn $a:ident,$b:ident) => {
        impl livestonk::Resolve<dyn $a> for livestonk::Livestonk {
            fn resolve() -> Box<dyn $a> {
                livestonk::Livestonk::resolve() as Box<$b>
            }
        }
    };
}
