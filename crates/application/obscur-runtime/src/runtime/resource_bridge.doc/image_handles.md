The [`ResourceHandle`] minted for each requested image URL, retained so the
hub lifecycle (begin_load → complete/fail) is driven per resource and a URL
is requested at most once.
