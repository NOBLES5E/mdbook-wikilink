let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
  mdbook-wikilink = import ./default.nix;
in pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    mdbook-wikilink
  ];
}
