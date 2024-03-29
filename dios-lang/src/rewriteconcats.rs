use itertools::Itertools;
use std::io;

// Rewrite a list of expressions to a nested concatenation of fixed-width
// vectors, based on the configuration's vector width
pub fn list_to_concats(vec_width: usize, input: &str) -> io::Result<String> {
    let v = lexpr::from_str(input)?;
    if let lexpr::Value::Cons(c) = v {
        let (mut list, _) = c.into_vec();
        // Remove the "list" from the start of the vec.
        list.remove(0);
        let mut concats = list
            .into_iter()
            .chunks(vec_width)
            .into_iter()
            .map(|c| {
                let mut chunk = c.into_iter().collect_vec();
                let buffer = vec_width - chunk.len();
                for _ in 0..buffer {
                    chunk.push(lexpr::Value::Number(lexpr::Number::from(0)))
                }
                chunk.insert(0, lexpr::Value::symbol("Vec"));
                lexpr::Value::list(chunk)
            })
            .collect_vec();
        concats.reverse();
        let init = concats.remove(0);
        let expr = concats.into_iter().fold(init, |acc, x| {
            lexpr::Value::list(vec![lexpr::Value::symbol("Concat"), x, acc])
        });
        lexpr::to_string(&expr)
    } else {
        panic!("Cannot rewrite non-list s-expression.")
    }
}
