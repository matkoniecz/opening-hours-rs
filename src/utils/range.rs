use std::cmp::{max, min, Ordering};
use std::fmt::Debug;
use std::ops::{Range, RangeInclusive};

use chrono::NaiveDateTime;
use opening_hours_syntax::rules::RuleKind;
use opening_hours_syntax::sorted_vec::UniqueSortedVec;

// DateTimeRange

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateTimeRange<'c, D> {
    pub range: Range<D>,
    pub kind: RuleKind,
    pub comments: UniqueSortedVec<&'c str>,
}

impl<'c, D: Debug + Eq> DateTimeRange<'c, D> {
    pub(crate) fn new_with_sorted_comments(
        range: Range<D>,
        kind: RuleKind,
        comments: UniqueSortedVec<&'c str>,
    ) -> Self {
        Self { range, kind, comments }
    }

    // TODO: doc
    pub fn comments(&self) -> &[&'c str] {
        &self.comments
    }

    // TODO: doc
    pub fn into_comments(self) -> UniqueSortedVec<&'c str> {
        self.comments
    }

    // TODO: doc
    pub fn map<E: Debug + Eq, F: Fn(D) -> E>(self, f: F) -> DateTimeRange<'c, E> {
        DateTimeRange {
            range: f(self.range.start)..f(self.range.end),
            kind: self.kind,
            comments: self.comments,
        }
    }
}

// WrappingRange

pub(crate) trait WrappingRange<T> {
    fn wrapping_contains(&self, elt: &T) -> bool;
}

impl<T: PartialOrd> WrappingRange<T> for RangeInclusive<T> {
    fn wrapping_contains(&self, elt: &T) -> bool {
        if self.start() <= self.end() {
            self.contains(elt)
        } else {
            self.start() <= elt || elt <= self.end()
        }
    }
}

// RangeCompare

pub(crate) trait RangeExt<T> {
    fn compare(&self, elt: &T) -> Ordering;
}

impl<T: PartialOrd> RangeExt<T> for RangeInclusive<T> {
    fn compare(&self, elt: &T) -> Ordering {
        debug_assert!(self.start() <= self.end());

        if elt < self.start() {
            Ordering::Less
        } else if elt > self.end() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl<T: PartialOrd> RangeExt<T> for Range<T> {
    fn compare(&self, elt: &T) -> Ordering {
        debug_assert!(self.start <= self.end);

        if elt < &self.start {
            Ordering::Less
        } else if elt >= &self.end {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

// Range operations

pub(crate) fn ranges_union<T: Ord>(
    ranges: impl IntoIterator<Item = Range<T>>,
) -> impl Iterator<Item = Range<T>> {
    // TODO: we could gain performance by ensuring that range iterators are
    //       always sorted.
    let mut ranges: Vec<_> = ranges.into_iter().collect();
    ranges.sort_unstable_by(|r1, r2| r1.start.cmp(&r2.start));

    // Get ranges by increasing start
    let mut ranges = ranges.into_iter();
    let mut current_opt = ranges.next();

    std::iter::from_fn(move || {
        if let Some(ref mut current) = current_opt {
            #[allow(clippy::while_let_on_iterator)]
            while let Some(item) = ranges.next() {
                if current.end >= item.start {
                    // The two intervals intersect with each other
                    if item.end > current.end {
                        current.end = item.end;
                    }
                } else {
                    return Some(current_opt.replace(item).unwrap());
                }
            }

            Some(current_opt.take().unwrap())
        } else {
            None
        }
    })
}

pub(crate) fn range_intersection<T: Ord>(range_1: Range<T>, range_2: Range<T>) -> Option<Range<T>> {
    let result = max(range_1.start, range_2.start)..min(range_1.end, range_2.end);

    if result.start < result.end {
        Some(result)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::{range_intersection, ranges_union};

    #[test]
    fn test_unions() {
        assert_eq!(
            &ranges_union([1..5, 0..1, 3..7, 8..9]).collect::<Vec<_>>(),
            &[0..7, 8..9]
        );
    }

    #[test]
    fn test_intersection() {
        assert!(range_intersection(0..1, 1..2).is_none());
        assert_eq!(range_intersection(0..3, 1..2).unwrap(), 1..2);
        assert_eq!(range_intersection(0..3, 2..4).unwrap(), 2..3);
    }
}
