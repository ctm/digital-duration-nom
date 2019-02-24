#[macro_use]
extern crate nom;
#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate newtype_derive;

pub mod duration;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
