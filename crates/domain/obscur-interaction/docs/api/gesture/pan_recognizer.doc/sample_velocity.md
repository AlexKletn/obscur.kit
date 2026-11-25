Folds one sample into the running velocity estimate.

A low-pass blend rather than the raw last-two-samples slope: touch
delivery jitters (two samples one millisecond apart, then one after
twenty), and the instantaneous slope across the short gap can be several
times the speed the finger is actually moving. Smoothing keeps a flick's
launch speed close to what the eye saw.
