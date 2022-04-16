use std::{borrow::Borrow, fmt::Display, num::ParseIntError, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Index(Vec<usize>);

impl Index {
    pub fn join(&self, value: usize) -> Index {
        let mut index = self.clone();
        index.0.push(value);
        index
    }
    pub fn parent(&self) -> Option<Index> {
        let mut index = self.clone();
        index.0.pop()?;
        Some(index)
    }
    pub fn new(subindexes: Vec<usize>) -> Self {
        Self(subindexes)
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/",
            self.0
                .iter()
                .map(|subindex| subindex.to_string())
                .collect::<Vec<String>>()
                .join("/")
        )
    }
}

impl FromStr for Index {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let subindexes = s
            .split('/')
            .filter(|str| !str.is_empty())
            .map(str::parse)
            .collect::<Result<Vec<usize>, _>>()?;
        Ok(Index(subindexes))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Catalogue<T, U> {
    Item(T),
    List { data: U, list: Vec<Catalogue<T, U>> },
}

impl<T, U> Catalogue<T, U> {
    pub fn get<I: Borrow<Index>>(&self, path: I) -> Option<&Self> {
        self.get_inner(&mut path.borrow().0.iter())
    }

    // TODO: find cleaner way to specify the iterator

    fn get_inner<'a>(&self, path: &mut impl Iterator<Item = &'a usize>) -> Option<&Self> {
        if let Some(index) = path.next() {
            if let Catalogue::List { list, .. } = self {
                list.get(*index)?.get_inner(path)
            } else {
                None
            }
        } else {
            Some(self)
        }
    }
}

#[test]
fn test_index() {
    let index = Index(vec![0, 1, 1]);
    dbg!(index.to_string());

    assert_eq!(index.to_string().parse(), Ok(index));
    assert_eq!("/".parse(), Ok(Index(vec![])));
    assert_eq!("/1/3/2".parse(), Ok(Index(vec![1, 3, 2])));
    assert!("/a/1/3".parse::<Index>().is_err());
}

#[test]
fn test_catalogue() {
    let catalogue = Catalogue::List {
        data: "z",
        list: vec![
            Catalogue::List {
                data: "a",
                list: vec![Catalogue::Item(1), Catalogue::Item(4)],
            },
            Catalogue::List {
                data: "b",
                list: vec![
                    Catalogue::List {
                        data: "b",
                        list: vec![Catalogue::Item(32)],
                    },
                    Catalogue::Item(2),
                ],
            },
        ],
    };

    assert_eq!(
        catalogue.get(Index(vec![])).cloned(),
        Some(catalogue.clone())
    );
    assert_eq!(
        catalogue.get(Index(vec![0, 0])).cloned(),
        Some(Catalogue::Item(1))
    );
    assert_eq!(
        catalogue.get(Index(vec![0, 1])).cloned(),
        Some(Catalogue::Item(4))
    );
    assert_eq!(catalogue.get(Index(vec![0, 0, 0])).cloned(), None,);
}
