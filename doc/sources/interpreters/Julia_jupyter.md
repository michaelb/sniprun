## Julia jupyter

The setup for the julia_jupyter interpreter is quite convoluted:

Indeed, the Julia jupyter kernel MUST be started before Sniprun can run (and there is even a consequent delay since the kernel is so slow to start).

You should start a julia jupyter kernel with the following command: 
` jupyter-kernel --kernel=julia-1.5 --KernelManager.connection_file=$HOME/.cache/sniprun/julia_jupyter/kernel_sniprun.json`

(adapt to your XDG_CACHE location if you're on Mac)

You manage kernel startup AND shutodwn manually. Why? There is a terrible data race if sniprun does it. Python_jupyter works, julia doesn't. That's about it.

If you want to use another kernel location, post a feature request at github.com/michaelb/sniprun and i'll see what I can do.
