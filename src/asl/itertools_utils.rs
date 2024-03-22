pub fn try_find<T, E, P, I>(iter: I, predicate: P) -> Result<Option<T>, E>
    where
        I: Iterator<Item = T>,
        P: Fn(&T) -> Result<bool, E>,
{
    for v in iter {
        let p = predicate(&v);
        match p {
            Ok(matches) => {
                if matches {
                    return Ok(Some(v));
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    Ok(None)
}
