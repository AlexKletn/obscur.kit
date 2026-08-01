§3.5 Rules — "no busy loop".

After the first frame is presented, advancing the deterministic clock by
10 000 ms in 16 ms steps with no input, commit or resource event must present
**zero** further frames.
