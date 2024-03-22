pub fn try_find<T, E, P, I>(iter: I, predicate: P) -> Result<Option<T>, E>
where
    I: Iterator<Item = T>,
    P: Fn(&T) -> Result<bool, E>,
{
    // TODO: Use the iter().try_iter() method after it stabilizes: https://github.com/rust-lang/rust/issues/63178
    //       This is probably not portable or makes many wrong assumptions. It's good enough for now :)
    for val in iter {
        let result = predicate(&val);
        match result {
            Ok(matches) => {
                if matches {
                    return Ok(Some(val));
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    Ok(None)
}
