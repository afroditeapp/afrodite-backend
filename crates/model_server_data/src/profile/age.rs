use model::ProfileAge;



/// Profile search age range which min and max are in
/// inclusive range of `[18, 99]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProfileSearchAgeRangeValidated {
    min: ProfileAge,
    max: ProfileAge,
}

impl ProfileSearchAgeRangeValidated {
    /// New range from two values. Automatically orders the values.
    pub fn new(value1: ProfileAge, value2: ProfileAge) -> Self {
        if value1.value() <= value2.value() {
            Self {
                min: value1,
                max: value2,
            }
        } else {
            Self {
                min: value2,
                max: value1,
            }
        }
    }

    pub fn min(&self) -> ProfileAge {
        self.min
    }

    pub fn max(&self) -> ProfileAge {
        self.max
    }

    pub fn is_match(&self, age: ProfileAge) -> bool {
        age.value() >= self.min.value() && age.value() <= self.max.value()
    }
}
