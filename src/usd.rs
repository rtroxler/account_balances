use std::ops;
use std::fmt;

#[derive(Copy, Clone)]
pub struct USD {
    pub pennies: i64
}

impl USD {
    pub fn zero() -> USD {
        USD {
            pennies: 0
        }
    }

    // TODO: Handle invalid floats
    pub fn from_float(d: f64) -> USD {
        let pennies = (d * 100.0) as i64;

        USD {
            pennies: pennies
        }
    }

    pub fn from_pennies(pennies: i64) -> USD {
        USD {
            pennies: pennies
        }
    }

    pub fn inverse(&self) -> USD {
        USD {
            pennies: -self.pennies
        }
    }
}

impl ops::AddAssign for USD {
    fn add_assign(&mut self, rhs: USD) {
        *self = USD {
            pennies: self.pennies + rhs.pennies
        };
    }
}
impl ops::Add for USD {
    type Output = USD;
    fn add(self, rhs: USD) -> USD {
        USD {
            pennies: self.pennies + rhs.pennies
        }
    }
}

impl ops::SubAssign for USD {
    fn sub_assign(&mut self, rhs: USD) {
        *self = USD {
            pennies: self.pennies - rhs.pennies
        };
    }
}
impl ops::Sub for USD {
    type Output = USD;
    fn sub(self, rhs: USD) -> USD {
        USD {
            pennies: self.pennies - rhs.pennies
        }
    }
}

impl fmt::Debug for USD {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dollars = self.pennies / 100;
        let cents = self.pennies % 100;
        let sign = if self.pennies.is_positive() { String::from("$") } else { String::from("-$") };
        write!(f, "{}{}.{}", sign, dollars.abs(), cents)
    }
}
