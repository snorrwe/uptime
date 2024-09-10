dev:
    dune exec dashboard

init:
    opam install . --deps-only

build:
    dune build
