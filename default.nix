let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
in pkgs.rustPlatform.buildRustPackage rec {
  name = "mdbook-wikilink"; 
  src = ./.;
  Cmd = [ "mdbook-wikilink" ];
  cargoSha256 = "0mlasy0gbj9nchgkprpmsyrbwms4y92djclyyvmqwmi5ppyzjnh5";
}
