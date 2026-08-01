This frame's group-compositing overlay (`opacity` / `transform`), rebuilt
from the live timelines every frame because these values reach paint
without ever entering computed style (spec §4.17, NG-MOTION-01).
