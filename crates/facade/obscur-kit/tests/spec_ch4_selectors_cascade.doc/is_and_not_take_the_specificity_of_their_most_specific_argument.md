selectors-4 §15: "The specificity of an :is(), :not(), or :has() pseudo-class
is replaced by the specificity of the most specific complex selector in its
selector list argument." — with the spec's examples ":is(em, #foo) has a
specificity of (1,0,0)" and ":not(em, strong#foo) has a specificity of (1,0,1)".
