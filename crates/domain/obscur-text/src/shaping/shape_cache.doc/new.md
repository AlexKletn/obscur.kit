An empty cache holding at most `capacity` shaped paragraphs across its two
generations. A capacity below 2 is raised to 2: a generation that rotated
on every insertion would keep nothing.
