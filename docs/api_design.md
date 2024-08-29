
# API design ideas and thoughts

## Profile age handling without birthdate (2024-08-29)

- Change birthdate to be optional info.
- Save unix timestamp of when initial setup was completed and
  current profile age at that point.
- When editing the profile there is an allowed range of profile age which
  changes when years pass.
  - The server keeps that profile age valid.
    - Server can makes modifications to the user profile, so there needs to be
      sync version for that.
- The algorithm for age range
  - The initial age (initialAge) is paired with the year of initial
    setup completed (initialSetupYear).
  - Year difference (yearDifference = currentYear - initialSetupYear) is
    used for changing the range min and max.
    - Min value: initialAge + yearDifference - 1.
    - Max value: initialAge + yearDifference + 1.

### Example: initial setup after birthday
Birthday: 2020-01-01
Initial setup: 2020-02-01
Age on initial setup day: 25

#### Age range at initial setup

Min: 24
Max: 26

#### Age range when year changes to 2021

User's real age changes 26.

Min: 25
Max: 27

#### Age range when year changes to 2022

Profile age is changed automatically to 26.

Min: 26
Max: 28

### Example: initial setup before birthday
Birthday: 2020-03-01
Initial setup: 2020-02-01
Age on initial setup day: 24

#### Age range at initial setup

Min: 23
Max: 25

User can change profile's age to 25 (the new real age after birthdate).

#### Age range when year changes to 2021

Min: 24
Max: 26

#### Age range when year changes to 2022

Profile age is changed automatically to 25.

Min: 25
Max: 27

### Example: why initialAge does not work for the min value

Birthday: 2020-02-01
Initial setup: 2020-03-01
Age on initial setup day: 25

#### Age range at initial setup

Min: 25
Max: 26

#### Age range when year changes to 2021

The profile age is changed too early 2021-01-01 to 26. User's real age changes
on 2021-02-01 to 26.

Min: 26
Max: 27
