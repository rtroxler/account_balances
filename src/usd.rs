use std::fmt;

pub struct USD {
    pub pennies: u64
}

impl USD {
    // TODO: Handle invalid floats
    pub fn from_float(d: f64) -> USD {
        let pennies = (d * 100.0) as u64;

        USD {
            pennies: pennies
        }
    }

    pub fn from_pennies(pennies: u64) -> USD {
        USD {
            pennies: pennies
        }
    }
}

impl fmt::Debug for USD {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dollars = self.pennies / 100;
        let cents = self.pennies % 100;
        write!(f, "${}.{}", dollars, cents)
    }
}
