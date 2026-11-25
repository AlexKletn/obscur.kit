Stable component id for the button widget.

Widgets share the `0x00D0_00NN` id space, one fixed value per widget so
the style/reconcile layer can map them without a name lookup:
button=`..01`, input=`..02`, checkbox=`..03`, select=`..04`.
