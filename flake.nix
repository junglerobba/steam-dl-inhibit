{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flakelight.url = "github:nix-community/flakelight";
    flakelight.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      flakelight,
      ...
    }@inputs:
    flakelight ./. rec {
      inherit inputs;
      package = import ./default.nix;
      withOverlays = [
        (final: prev: { steam-dl-inhibit = package; })
      ];
      devShell = import ./shell.nix;
    };
}
