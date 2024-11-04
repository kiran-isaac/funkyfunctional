mod types;

pub fn add(left: i64, right: i64) -> i64 {
    left + right
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
