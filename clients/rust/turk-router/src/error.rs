use core::fmt;

/// Why an instruction could not be built. Each variant names an input the program would refuse
/// before reaching any named error of its own, so this crate refuses it first, by name.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A byte that names no menu kind.
    UnknownHopKind {
        /// The byte given.
        raw: u8,
    },
    /// The route mint list is empty.
    NoRouteMints,
    /// More route mints than the wire can carry.
    TooManyRouteMints {
        /// How many were given.
        given: usize,
        /// The wire's ceiling.
        max: usize,
    },
    /// The menu is empty.
    EmptyMenu,
    /// More menu pools than the wire can carry.
    TooManyMenuPools {
        /// How many were given.
        given: usize,
        /// The wire's ceiling.
        max: usize,
    },
    /// The menu's declared accounts, summed, exceed the program's budget.
    MenuAccountBudgetExceeded {
        /// The sum of every window's account count.
        declared: usize,
        /// The program's budget.
        budget: usize,
    },
    /// A venue's variable tail was given outside the length its instruction accepts.
    TailLength {
        /// How many accounts were given.
        given: usize,
        /// The shortest tail the venue accepts.
        min: usize,
        /// The longest tail the venue accepts.
        max: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownHopKind { raw } => {
                write!(f, "hop kind {raw} is not one the menu accepts")
            }
            Error::NoRouteMints => write!(f, "a route needs at least one route mint"),
            Error::TooManyRouteMints { given, max } => {
                write!(
                    f,
                    "{given} route mints given, the wire carries at most {max}"
                )
            }
            Error::EmptyMenu => write!(f, "a menu needs at least one pool"),
            Error::TooManyMenuPools { given, max } => {
                write!(
                    f,
                    "{given} menu pools given, the wire carries at most {max}"
                )
            }
            Error::MenuAccountBudgetExceeded { declared, budget } => write!(
                f,
                "the menu declares {declared} accounts, the program budgets {budget}"
            ),
            Error::TailLength { given, min, max } => {
                write!(
                    f,
                    "{given} tail accounts given, the venue accepts {min}..={max}"
                )
            }
        }
    }
}

impl std::error::Error for Error {}
