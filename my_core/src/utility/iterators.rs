#[derive(Debug)]
pub(crate) struct Pair<T1, T2>(T1, T2);

pub(crate) struct PairIter<'a, T1, T2> {
    pub(crate) a_iter: std::slice::Iter<'a, T1>,
    pub(crate) b_iter: std::slice::Iter<'a, T2>,
    current:           Option<Pair<&'a T1, &'a T2>>,
}

impl<'a, T1, T2> PairIter<'a, T1, T2> {
    pub(crate) fn new(a: std::slice::Iter<'a, T1>, b: std::slice::Iter<'a, T2>) -> Self {
        Self {
            a_iter:  a,
            b_iter:  b,
            current: None,
        }
    }
}

impl<'a, T1, T2> Iterator for PairIter<'a, T1, T2> {
    type Item = &'a Pair<&'a T1, &'a T2>;

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(x), Some(y)) = (self.a_iter.next(), self.b_iter.next()) {
            self.current = Some(Pair(x, y));
            unsafe { std::mem::transmute(self.current.as_ref()) }
        } else {
            None
        }
    }
}
