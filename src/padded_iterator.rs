pub(crate) struct PaddedIterator<Item, Values> {
    padding_value: Option<Item>,
    padding_remaining: usize,
    padding_before_values: bool,
    values: Values,
}

impl<Item: Copy, Values> PaddedIterator<Item, Values> {
    pub(crate) fn new(
        values: Values,
        padding_value: Option<Item>,
        missing: usize,
        padding_before_values: bool,
    ) -> Self {
        Self {
            padding_value,
            padding_remaining: if padding_value.is_some() { missing } else { 0 },
            padding_before_values,
            values,
        }
    }
}

impl<Item: Copy, Values> Iterator for PaddedIterator<Item, Values>
where
    Values: ExactSizeIterator<Item = Item>,
{
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.padding_before_values && 0 < self.padding_remaining {
            self.padding_remaining -= 1;
            return self.padding_value;
        }

        if let Some(value) = self.values.next() {
            return Some(value);
        }

        if 0 < self.padding_remaining {
            self.padding_remaining -= 1;
            return self.padding_value;
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.padding_remaining + self.values.len();
        (len, Some(len))
    }
}

impl<Item: Copy, Values> ExactSizeIterator for PaddedIterator<Item, Values> where
    Values: ExactSizeIterator<Item = Item>
{
}

impl<Item: Copy, Values> core::iter::FusedIterator for PaddedIterator<Item, Values> where
    Values: ExactSizeIterator<Item = Item> + core::iter::FusedIterator
{
}
