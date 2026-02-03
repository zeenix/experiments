# Overlay to enable tokio features needed for this PoC
features += ['rt', 'rt-multi-thread', 'time', 'macros']
extra_args += [
  '--cfg', 'feature="rt"',
  '--cfg', 'feature="rt-multi-thread"',
  '--cfg', 'feature="time"',
  '--cfg', 'feature="macros"',
]
tokio_macros_dep = dependency('tokio-macros-2-rs')
extra_deps += [tokio_macros_dep]
