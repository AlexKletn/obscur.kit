§7.2: "- gender;". Guards the *value* layer the message layer selects on:
`specs/cldr/grammaticalFeatures.xml` line 216 gives `ru`
`<grammaticalGender values="masculine feminine neuter"/>` and line 220 gives
`pl` `<grammaticalGender values="animate inanimate personal feminine neuter"/>`,
so a gender value is a *name*, and `personal` must be as expressible as
`masculine`.
