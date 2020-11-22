#![allow(dead_code)]
mod bot;
mod dag;
mod data;
mod strategy;
mod time_series;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
