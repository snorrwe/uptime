{
  description = "devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            ocamlformat_0_22_4
            ocaml
            opam
            just
            pkg-config
            # dream deps
            openssl
            gmp
            libev
          ];
          shellHook = ''
            eval $(opam env)
          '';
        };
      }
    );
}
