use crate::parser::Pickable;

#[derive(Debug, Default)]
#[repr(u8)]
pub enum Yes {
    Yes,
    #[default]
    No,
}

impl From<bool> for Yes {
    fn from(b: bool) -> Self {
        if b { Yes::Yes } else { Yes::No }
    }
}

impl From<Yes> for bool {
    fn from(val: Yes) -> Self {
        match val {
            Yes::Yes => true,
            Yes::No => false,
        }
    }
}

impl std::ops::Deref for Yes {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        static TRUE: bool = true;
        static FALSE: bool = false;
        match self {
            Yes::Yes => &TRUE,
            Yes::No => &FALSE,
        }
    }
}

impl Yes {
    pub fn is_yes(&self) -> bool {
        matches!(self, Yes::Yes)
    }

    pub fn is_no(&self) -> bool {
        matches!(self, Yes::No)
    }
}

impl Pickable for Yes {
    type Output = Yes;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        let value = pick_bool(args, flag, &["y", "yes"]);
        Some(value.into())
    }
}

#[derive(Debug, Default)]
#[repr(u8)]
pub enum True {
    True,
    #[default]
    False,
}

impl From<bool> for True {
    fn from(b: bool) -> Self {
        if b { True::True } else { True::False }
    }
}

impl From<True> for bool {
    fn from(val: True) -> Self {
        match val {
            True::True => true,
            True::False => false,
        }
    }
}

impl std::ops::Deref for True {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        static TRUE: bool = true;
        static FALSE: bool = false;
        match self {
            True::True => &TRUE,
            True::False => &FALSE,
        }
    }
}

impl True {
    pub fn is_true(&self) -> bool {
        matches!(self, True::True)
    }

    pub fn is_false(&self) -> bool {
        matches!(self, True::False)
    }
}

impl Pickable for True {
    type Output = True;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        let value = pick_bool(args, flag, &["true", "t"]);
        Some(value.into())
    }
}

fn pick_bool(
    args: &mut crate::parser::Argument,
    flag: mingling_core::Flag,
    positive: &[&str],
) -> bool {
    let content = args.pick_argument(flag);
    match content {
        Some(content) => {
            let s = content.as_str();
            positive.contains(&s)
        }
        None => false,
    }
}
