pub fn partition<I, F, O>(i: I, pred: F) -> (O, O)
where
  I: Clone + IntoIterator,
  F: Fn(&I::Item) -> bool,
  O: FromIterator<I::Item>,
{
  let left = i.clone().into_iter().filter(|x| !(pred(x))).collect::<O>();
  let right = i.into_iter().filter(pred).collect::<O>();
  (left, right)
}
