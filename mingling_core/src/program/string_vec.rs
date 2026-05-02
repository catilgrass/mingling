#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct StringVec {
    vec: Vec<String>,
}

impl std::ops::Deref for StringVec {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl From<StringVec> for Vec<String> {
    fn from(val: StringVec) -> Self {
        val.vec
    }
}

impl<const N: usize> From<[&str; N]> for StringVec {
    fn from(slice: [&str; N]) -> Self {
        StringVec {
            vec: slice.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

impl From<&[&str]> for StringVec {
    fn from(slice: &[&str]) -> Self {
        StringVec {
            vec: slice.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

impl From<Vec<String>> for StringVec {
    fn from(vec: Vec<String>) -> Self {
        StringVec { vec }
    }
}

impl From<&[String]> for StringVec {
    fn from(slice: &[String]) -> Self {
        StringVec {
            vec: slice.to_vec(),
        }
    }
}

impl From<Vec<&str>> for StringVec {
    fn from(vec: Vec<&str>) -> Self {
        StringVec {
            vec: vec.iter().map(|&s| s.to_string()).collect(),
        }
    }
}
