#[derive(Debug, PartialEq, Eq)]
pub enum Child {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrefixOrd {
    Same,
    Contains,
    IsContained,
    Disjoint,
}

pub trait Cmp<P: super::Prefix> {
    fn containership(&self, longer: &P) -> (PrefixOrd, u8, Option<Child>);
    fn cmp(&self, b: &P) -> (PrefixOrd, bool, u8, Option<Child>);
}

impl<P, T> Cmp<T> for P
where
    P: super::Prefix,
    T: super::Prefix,
{
    // helper which compares to see if self contains the longer prefix.
    //
    // It assumes that self.length() <= longer.length(). Otherwise, the behavior is undefined.
    //
    // `ord`:    how self relates to the longer prefix (Same, Contains, Disjoint).
    //           since self is shorter than longer, IsContained is not possible.
    // `common`: number of leading bits that are equal in the two up to the shorter mask length.
    // `child`:  tells whether the first non-common bit in `longer` is a 0 (left) or 1 (right).
    //           It is only relevant if `exact` is false.
    //
    //  The following table describes how to interpret results:
    //
    // | ord      | common | child | note
    // |----------|--------|-------|-------
    // | Disjoint | 0..31  | Left  | the two are disjoint and `longer` compares less than `shorter`
    // | Disjoint | 0..31  | Right | the two are disjoint and `longer` compares greater than `shorter`
    // | Contains | 0..31  | Left  | `longer` should be `shorter`'s left child
    // | Contains | 0..31  | Right | `longer` should be `shorter`'s right child
    // | Same     | 0..32  | None  | `shorter` and `longer` are the same prefix
    fn containership(&self, longer: &T) -> (PrefixOrd, u8, Option<Child>) {
        use super::Address;

        let short: [u8; 4] = self.address().octets();
        let long: [u8; 4] = longer.address().octets();

        for i in 0..4 {
            let offset = (i * 8) as u8;
            let short_len = self.length() - offset;
            let common = std::cmp::min(short_len, (short[i] ^ long[i]).leading_zeros() as u8);
            let ord = match short_len <= common {
                true => match short_len == longer.length() - offset {
                    true => break,
                    false => PrefixOrd::Contains,
                },
                false => match common == 8 {
                    true => continue,
                    false => PrefixOrd::Disjoint,
                },
            };
            let child = Some({
                let pivot_bit = match common == 8 {
                    true => 0x80 & long[i + 1],
                    false => 0x80 >> common & long[i],
                };
                match pivot_bit == 0 {
                    true => Child::Left,
                    false => Child::Right,
                }
            });
            return (ord, common + offset, child);
        }

        (PrefixOrd::Same, self.length(), None)
    }

    fn cmp(&self, other: &T) -> (PrefixOrd, bool, u8, Option<Child>) {
        let (reversed, (ord, common, child)) = match other.length() < self.length() {
            true => (true, other.containership(self)),
            false => (false, self.containership(other)),
        };
        let ord = match reversed && ord == PrefixOrd::Contains {
            true => PrefixOrd::IsContained,
            false => ord,
        };
        (ord, reversed, common, child)
    }
}

#[cfg(test)]
mod test {
    use super::super::Prefix;
    use super::*;

    pub fn p(s: &str) -> ipnet::Ipv4Net {
        s.parse().expect("bad prefix")
    }

    fn cmp(
        a: ipnet::Ipv4Net,
        b: ipnet::Ipv4Net,
        expected_ord: PrefixOrd,
        expected_common: u8,
        expected_child: Option<Child>,
    ) {
        let (ord, common, child) = Cmp::containership(&a, &b);
        assert_eq!(expected_ord, ord);
        assert_eq!(expected_common, common);
        assert_eq!(expected_child, child);

        // compare forward
        let (ord, reversed, common, child) = Cmp::cmp(&a, &b);
        assert!(!reversed);
        assert_eq!(expected_common, common);
        assert_eq!(expected_child, child);
        assert_eq!(expected_ord, ord);

        // compare reversed
        let (ord, reversed, common_, _) = Cmp::cmp(&b, &a);
        assert_eq!(a.length() != b.length(), reversed);
        assert_eq!(expected_common, common_);
        let expected_ord = match expected_ord {
            PrefixOrd::Contains => PrefixOrd::IsContained,
            PrefixOrd::IsContained => PrefixOrd::Contains,
            _ => expected_ord,
        };
        assert_eq!(expected_ord, ord);
    }

    runner::tests! { cmp {
        trivial(
            p("0.0.0.0/0"),
            p("0.0.0.0/0"),
            PrefixOrd::Same, 0, None);
        exact(
            p("10.0.0.0/16"),
            p("10.0.0.0/16"),
            PrefixOrd::Same, 16, None);
        exact_partial(
            p("10.0.0.0/19"),
            p("10.0.31.0/19"),
            PrefixOrd::Same, 19, None);
        empty_prefix_match(
            p("0.0.0.0/0"),
            p("10.10.0.0/16"),
            PrefixOrd::Contains, 0, Some(Child::Left));
        empty_prefix_match_backwards(
            p("0.0.0.0/0"),
            p("130.10.0.0/16"),
            PrefixOrd::Contains, 0, Some(Child::Right));
        matches(
            p("10.0.0.0/8"),
            p("10.10.0.0/16"),
            PrefixOrd::Contains, 8, Some(Child::Left));
        matches_partial(
            p("10.200.0.0/9"),
            p("10.129.0.0/16"),
            PrefixOrd::Contains, 9, Some(Child::Left));
        matches_backwards(
            p("10.0.0.0/8"),
            p("10.200.0.0/16"),
            PrefixOrd::Contains, 8, Some(Child::Right));
        matches_backwards_partial(
            p("10.240.0.0/9"),
            p("10.200.0.0/16"),
            PrefixOrd::Contains, 9, Some(Child::Right));
        disjoint(
            p("0.0.0.0/1"),
            p("128.0.0.0/1"),
            PrefixOrd::Disjoint, 0, Some(Child::Right));
        disjoint_longer(
            p("0.0.0.0/17"),
            p("0.0.128.0/17"),
            PrefixOrd::Disjoint, 16, Some(Child::Right));
        disjoint_longer_partial(
            p("0.0.0.0/17"),
            p("0.1.0.0/17"),
            PrefixOrd::Disjoint, 15, Some(Child::Right));
        disjoint_backwards(
            p("128.0.0.0/1"),
            p("0.0.0.0/1"),
            PrefixOrd::Disjoint, 0, Some(Child::Left));
        disjoint_backwards_longer(
            p("0.0.128.0/19"),
            p("0.0.0.0/19"),
            PrefixOrd::Disjoint, 16, Some(Child::Left));
        disjoint_backwards_longer_partial(
            p("0.1.0.0/19"),
            p("0.0.0.0/19"),
            PrefixOrd::Disjoint, 15, Some(Child::Left));
        disjoint_with_common(
            p("10.0.0.0/16"),
            p("10.10.0.0/16"),
            PrefixOrd::Disjoint, 12, Some(Child::Right));
        disjoint_with_more_disjoint_bytes(
            p("0.255.255.0/24"),
            p("128.0.0.0/24"),
            PrefixOrd::Disjoint, 0, Some(Child::Right));
    } }
}
